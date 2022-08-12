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

use super::{
	error::BulkError,
	task::{submit_job, with_db, TaskInput},
};
use check_if_email_exists::CheckEmailInputProxy;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::cmp::min;
use warp::Filter;

// this configures the number of emails passed to every task
// this can be configured but will require changes in the
// in the `crate::check::check_email` function which assumes a task can have
// only one email. This will also require changing the
// email_verification_task itself to handle multiple
// outputs and commit them to the database.
const EMAIL_TASK_BATCH_SIZE: usize = 1;

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateBulkRequestBody {
	input_type: String,
	input: Vec<String>,
	proxy: Option<CheckEmailInputProxy>,
	hello_name: Option<String>,
	from_email: Option<String>,
	smtp_ports: Option<Vec<u16>>,
}

struct CreateBulkRequestBodyIterator {
	body: CreateBulkRequestBody,
	index: usize,
	batch_size: usize,
}

impl IntoIterator for CreateBulkRequestBody {
	type Item = TaskInput;
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
	type Item = TaskInput;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.body.input.len() {
			let bounded_index = min(self.index + self.batch_size, self.body.input.len());
			let to_emails = self.body.input[self.index..bounded_index].to_vec();
			let item = TaskInput {
				to_emails,
				smtp_ports: self.body.smtp_ports.clone().unwrap_or_else(|| vec![25]),
				proxy: self.body.proxy.clone(),
				hello_name: self.body.hello_name.clone(),
				from_email: self.body.from_email.clone(),
			};

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

/// handles input, creates db entry for job and tasks for verification
async fn create_bulk_request(
	body: CreateBulkRequestBody,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	if body.input.is_empty() {
		return Err(BulkError::EmptyInput.into());
	}

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
			target: "reacher",
			"Failed to create job record for [body={:?}] with [error={}]",
			&body,
			e
		);
		BulkError::from(e)
	})?;

	for task_input in body.into_iter() {
		let task_uuid = submit_job(&conn_pool, rec.id, task_input).await?;

		log::debug!(
			target: "reacher",
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
pub fn create_bulk_job(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk")
		.and(warp::post())
		.and(with_db(o))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(
			move |conn_pool: Pool<Postgres>, body: CreateBulkRequestBody| {
				create_bulk_request(body, conn_pool.clone())
			},
		)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
