//! This file implements the `POST /bulk` endpoint.
use crate::{
	errors::ReacherError,
	routes::check_email::{header::check_header, post::RetryOption},
};
use async_recursion::async_recursion;
use check_if_email_exists::{
	check_email as ciee_check_email, CheckEmailInput, CheckEmailInputProxy, CheckEmailOutput,
	Reachable,
};
use sqlx::{Pool, Postgres};
use warp::Filter;

use std::{collections::HashMap, error::Error};

use crate::routes::check_email::known_errors;
use serde::{Deserialize, Serialize};
use sqlxmq::{job, CurrentJob};

const EMAIL_TASK_BATCH_SIZE: usize = 1;

/// Errors that can happen during an email verification.
#[derive(Debug)]
enum CheckEmailError {
	/// We get an `is_reachable` Unknown. We consider this internally as an
	/// error case, so that we can do retry mechanisms (see select_ok & retry).
	Unknown((CheckEmailOutput, RetryOption)),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskInput {
	job_id: i32,
	input: CheckEmailInput,
}

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CreateBulkRequestBody {
	input_type: String,
	input: Vec<String>,
	proxy: Option<CheckEmailInputProxy>,
	hello_name: Option<String>,
	from_email: Option<String>,
	smtp_port: Option<u16>,
}

pub struct CreateBulkRequestBodyIterator {
	body: CreateBulkRequestBody,
	index: usize,
	batch_size: usize,
}

impl IntoIterator for CreateBulkRequestBody {
	type Item = CreateBulkRequestBody;
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
	type Item = CreateBulkRequestBody;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index <= self.body.input.len() {
			let item = Some(CreateBulkRequestBody {
				input_type: self.body.input_type.clone(),
				input: self.body.input[self.index..self.index + self.batch_size].to_vec(),
				proxy: self.body.proxy.clone(),
				hello_name: self.body.hello_name.clone(),
				from_email: self.body.from_email.clone(),
				smtp_port: self.body.smtp_port.clone(),
			});

			self.index = self.index + self.batch_size;

			item
		} else {
			None
		}
	}
}

impl Into<CheckEmailInput> for CreateBulkRequestBody {
	fn into(self) -> CheckEmailInput {
		let mut input = CheckEmailInput::new(self.input);

		if let Some(name) = self.hello_name {
			input.set_hello_name(name);
		}

		if let Some(email) = self.from_email {
			input.set_from_email(email);
		}

		if let Some(port) = self.smtp_port {
			input.set_smtp_port(port as u16);
		}

		if let Some(proxy) = self.proxy {
			input.set_proxy(CheckEmailInputProxy {
				host: proxy.get("host").unwrap().clone(),
				port: proxy.get("port").unwrap().parse().unwrap(),
			});
		}

		input
	}
}

/// Endpoint response body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateBulkResponseBody {
	job_id: i32,
}

// Arguments to the `#[job]` attribute allow setting default job options.
/// This task tries to verify the given email and inserts the results
/// into the email verification db table
#[job]
pub async fn email_verification_task(
	mut current_job: CurrentJob,
	// Additional arguments are optional, but can be used to access context
	// provided via [`JobRegistry::set_context`].
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	let task_input: TaskInput = current_job.json()?.unwrap();

	// Retry each future twice, to avoid grey-listing.
	if let Ok((response, _)) = retry(&task_input.input, RetryOption::Direct, 2).await {
		let rec = sqlx::query!(
			r#"
			INSERT INTO email_results (job_id, result)
			VALUES ($1, $2)
			"#,
			task_input.job_id,
			serde_json::json!(response)
		);
	} else {
		log::debug!(
			target:"reacher",
			"Failed [email={}] for [job={}]",
			task_input.job_id,
			task_input.input.to_emails[0]
		);
	}

	current_job.complete().await?;

	Ok(())
}

/// Retry the check ciee_check_email function, in particular to avoid
/// greylisting.
#[async_recursion]
async fn retry(
	input: &CheckEmailInput,
	retry_option: RetryOption,
	count: usize,
) -> Result<(CheckEmailOutput, RetryOption), CheckEmailError> {
	log::debug!(
		target:"reacher",
		"[email={}] Checking with retry option {}, attempt #{}",
		input.to_emails[0],
		retry_option,
		count,
	);

	let result = ciee_check_email(input)
		.await
		.pop()
		.expect("Input contains one email, so does output. qed.");

	log::debug!(
		target:"reacher",
		"[email={}] Got result with retry option {}, attempt #{}, is_reachable={:?}",
		input.to_emails[0],
		retry_option,
		count,
		result.is_reachable
	);

	// If we get an unknown error, log it.
	known_errors::log_unknown_errors(&result, retry_option);

	if result.is_reachable == Reachable::Unknown {
		if count <= 1 {
			Err(CheckEmailError::Unknown((result, retry_option)))
		} else {
			retry(input, retry_option, count - 1).await
		}
	} else {
		Ok((result, retry_option))
	}
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
			target:"reacher/v0/bulk/",
			"Failed to create job record for [body={:?}] with [error={}]",
			&body,
			e
		);
		ReacherError::from(e)
	})?;

	for job in body.into_iter() {
		let task_input = TaskInput {
			input: job.into(),
			job_id: rec.id,
		};
		// TODO handle errors gracefully
		email_verification_task
			.builder()
			.set_json(&task_input)
			.unwrap()
			.spawn(&conn_pool)
			.await
			.unwrap();
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
		.and(check_header())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(move |body: CreateBulkRequestBody| create_bulk_request(body, conn_pool.clone()))
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
