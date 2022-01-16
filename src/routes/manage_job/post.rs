//! This file implements the `POST /bulk` endpoint.
use crate::routes::check_email::header::check_header;
use sqlx::{Pool, Postgres};
use warp::Filter;

use std::{
	error::Error,
};

use serde::{Deserialize, Serialize};
use sqlxmq::{job, CurrentJob};

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloRequest {
	name: Option<String>,
}

// Arguments to the `#[job]` attribute allow setting default job options.
#[job(channel_name = "foo")]
pub async fn example_job(
	// The first argument should always be the current job.
	mut current_job: CurrentJob,
	// Additional arguments are optional, but can be used to access context
	// provided via [`JobRegistry::set_context`].
	message: &'static str,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	// Decode a JSON payload
	let who: Option<String> = current_job.json()?;

	// Do some work
	println!("{}, {}!", message, who.as_deref().unwrap_or("world"));

	// Mark the job as complete
	current_job.complete().await?;

	Ok(())
}

/// The main `check_email` function that implements the logic of this route.
async fn create_bulk_email_job(
	_: HelloRequest,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	if let Ok(uuid) = example_job.builder().spawn(&conn_pool).await {
		Ok(warp::reply::with_status(
			uuid.to_string(),
			warp::http::StatusCode::CREATED,
		))
	} else {
		Ok(warp::reply::with_status(
			"Unable to create job".to_string(),
			warp::http::StatusCode::INTERNAL_SERVER_ERROR,
		))
	}
}

/// Create the `POST /bulk` endpoint.
/// The endpoint accepts list of email address and creates
/// a new job to check them.
pub fn post_check_bulk_email_job(
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
		.and_then(move |body: HelloRequest| create_bulk_email_job(body, conn_pool.clone()))
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
