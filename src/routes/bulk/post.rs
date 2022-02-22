// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This file implements the `POST /bulk` endpoint.

use crate::check::{check_email, SMTP_TIMEOUT};
use crate::errors::ReacherError;
use check_if_email_exists::{CheckEmailInput, CheckEmailInputProxy};
use sqlx::{Pool, Postgres};
use std::{cmp::min, error::Error, time::Duration};
use warp::Filter;

use serde::{Deserialize, Serialize};
use sqlxmq::{job, CurrentJob};

// this configures the number of emails passed to every task
// this can be configured but will require changes in the
// in the `crate::check::check_email` function which assumes a task can have
// only one email. This will also require changing the
// email_verification_task itself to handle multiple
// outputs and commit them to the database.
const EMAIL_TASK_BATCH_SIZE: usize = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TaskInput {
	job_id: i32,
	input: CheckEmailInput,
}

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateBulkRequestBody {
	input_type: String,
	input: Vec<String>,
	proxy: Option<CheckEmailInputProxy>,
	hello_name: Option<String>,
	from_email: Option<String>,
	smtp_port: Option<u16>,
}

struct CreateBulkRequestBodyIterator {
	body: CreateBulkRequestBody,
	index: usize,
	batch_size: usize,
}

impl IntoIterator for CreateBulkRequestBody {
	type Item = CheckEmailInput;
	type IntoIter = CreateBulkRequestBodyIterator;

	fn into_iter(self) -> Self::IntoIter {
		CreateBulkRequestBodyIterator {
			body: self,
			index: 0,
			batch_size: EMAIL_TASK_BATCH_SIZE,
		}
	}
}

impl Iterator for CreateBulkRequestBodyIterator {
	type Item = CheckEmailInput;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.body.input.len() {
			let bounded_index = min(self.index + self.batch_size, self.body.input.len());
			let to_emails = self.body.input[self.index..bounded_index].to_vec();
			let mut item = CheckEmailInput::new(to_emails);

			if let Some(name) = &self.body.hello_name {
				item.set_hello_name(name.clone());
			}

			if let Some(email) = &self.body.from_email {
				item.set_from_email(email.clone());
			}

			if let Some(port) = self.body.smtp_port {
				item.set_smtp_port(port);
			}

			if let Some(proxy) = &self.body.proxy {
				item.set_proxy(proxy.clone());
			}

			item.set_smtp_timeout(Duration::from_secs(SMTP_TIMEOUT));

			self.index = bounded_index;

			Some(item)
		} else {
			None
		}
	}
}

/// Endpoint response body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateBulkResponseBody {
	job_id: i32,
}

/// Arguments to the `#[job]` attribute allow setting default job options.
/// This task tries to verify the given email and inserts the results
/// into the email verification db table
// NOTE: if EMAIL_TASK_BATCH_SIZE is made greater than 1 this logic
// will have to be changed to handle a vector outputs from `check_email`.
#[job]
pub async fn email_verification_task(
	mut current_job: CurrentJob,
	// Additional arguments are optional, but can be used to access context
	// provided via [`JobRegistry::set_context`].
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	let task_input: TaskInput = current_job.json()?.unwrap();

	log::debug!(
		target:"reacher",
		"Starting task [email={}] for [job_id={}] and [uuid={}]",
		task_input.input.to_emails[0],
		task_input.job_id,
		current_job.id(),
	);

	let response = check_email(&task_input.input).await;

	log::debug!(
		target:"reacher",
		"Got task result [email={}] for [job_id={}] and [uuid={}] with [is_reachable={:?}]",
		task_input.input.to_emails[0],
		task_input.job_id,
		current_job.id(),
		response.is_reachable,
	);

	#[allow(unused_variables)]
	let rec = sqlx::query!(
		r#"
			INSERT INTO email_results (job_id, result)
			VALUES ($1, $2)
			"#,
		task_input.job_id,
		serde_json::json!(response)
	)
	// TODO: This is a simplified solution and will work when
	// the task queue and email results tables are in the same
	// database. Keeping them in separate database will require
	// some custom logic on the job registry side
	// https://github.com/Diggsey/sqlxmq/issues/4
	.fetch_optional(current_job.pool())
	.await
	.map_err(|e| {
		log::error!(
			target:"reacher",
			"Failed to write [email={}] result to db for [job_id={}] and [uuid={}] with [error={}]",
			task_input.input.to_emails[0],
			task_input.job_id,
			current_job.id(),
			e
		);

		e
	})?;

	log::debug!(
		target:"reacher",
		"Wrote result for [email={}] for [job_id={}] and [uuid={}]",
		task_input.input.to_emails[0],
		task_input.job_id,
		current_job.id(),
	);

	current_job.complete().await?;

	Ok(())
}

/// handles input, creates db entry for job and tasks for verification
async fn create_bulk_request(
	body: CreateBulkRequestBody,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	// create job entry
	let rec = sqlx::query!(
		r#"
		INSERT INTO bulk_jobs (total_records)
		VALUES ($1)
		RETURNING id
		"#,
		body.input.len() as i32
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target:"reacher",
			"Failed to create job record for [body={:?}] with [error={}]",
			&body,
			e
		);
		ReacherError::from(e)
	})?;

	for task_input in body.into_iter() {
		let task = TaskInput {
			input: task_input,
			job_id: rec.id,
		};

		let task_uuid = email_verification_task
			.builder()
			.set_json(&task)
			.unwrap()
			.spawn(&conn_pool)
			.await
			.map_err(|e| {
				log::error!(
					target:"reacher",
					"Failed to submit task for [job={}] with [error={}]",
					rec.id,
					e
				);

				ReacherError::from(e)
			})?;

		log::debug!(
			target:"reacher",
			"Submitted task to sqlxmq for [job={}] with [uuid={}]",
			rec.id,
			task_uuid
		);
	}

	Ok(warp::reply::json(&CreateBulkResponseBody {
		job_id: rec.id,
	}))
}

/// Create the `POST /bulk` endpoint.
/// The endpoint accepts list of email address and creates
/// a new job to check them.
pub fn create_bulk_email_vrfy_job(
	conn_pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk")
		.and(warp::post())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(move |body: CreateBulkRequestBody| create_bulk_request(body, conn_pool.clone()))
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
