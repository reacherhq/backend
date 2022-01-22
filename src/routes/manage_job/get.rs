use crate::routes::check_email::header::check_header;
use sqlx::{Pool, Postgres};
use warp::Filter;

use serde::Serialize;

use check_if_email_exists::CheckEmailOutput;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;

#[derive(sqlx::Type, Debug, Serialize)]
#[sqlx(type_name = "valid_status", rename_all = "lowercase")]
pub enum ValidStatus {
	Running,
	Completed,
	Stopped,
}

/// Job record stores the information about a submitted job
///
/// `job_status` field is an update on read field. It's
/// status will be derived from counting number of
/// completed email verification tasks. It will be updated
/// with the most recent status of the job.
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct JobRecord {
	id: i32,
	created_at: DateTime<Utc>,
	total_records: i32,
	job_status: ValidStatus,
}

/// Email record stores the result of a completed email verification task
///
/// It related to it's parent job through job_id
/// It stores the result of a verification as jsonb field
/// serialized from `CheckEmailOutput`
#[derive(sqlx::FromRow, Debug)]
pub struct EmailRecord {
	job_id: i32,
	email_id: String,
	result: Json<CheckEmailOutput>,
}

async fn job_status(
	job_id: i32,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	let rec = sqlx::query_as!(
		JobRecord,
		r#"
		SELECT id, created_at, total_records, job_status as "job_status: _" FROM blk_vrfy_job
		WHERE id = $1
		LIMIT 1
		"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await;

	// TODO get aggregate info from other table
	// TODO Get and update job status from aggregate info

	match rec {
		Ok(rec) => Ok(warp::reply::json(&rec)),
		Err(err) => Ok(warp::reply::json(&"Record not found".to_string())),
	}
}

pub fn get_job_status(
	conn_pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk" / i32)
		.and(warp::get())
		.and(check_header())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		// TODO: Configure max size limit for a bulk job
		.and_then(move |job_id| job_status(job_id, conn_pool.clone()))
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
