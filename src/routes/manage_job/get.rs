use crate::routes::check_email::header::check_header;
use sqlx::{Pool, Postgres};
use warp::Filter;

use serde::Serialize;

use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(sqlx::Type, Debug, Serialize)]
#[sqlx(type_name = "valid_status", rename_all = "lowercase")]
pub enum ValidStatus {
	Pending,
	Running,
	Completed,
	Stopped,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct JobRecord {
	id: i32,
	job_uuid: Option<Uuid>,
	created_at: DateTime<Utc>,
	attempt_at: DateTime<Utc>,
	total_records: i32,
	total_processed: i32,
	summary_total_safe: i32,
	summary_total_invalid: i32,
	summary_total_risky: i32,
	summary_total_unknown: i32,
	job_status: ValidStatus,
}

#[derive(sqlx::FromRow, Debug)]
pub struct EmailRecord {
	job_id: i32,
	record_id: i32,
	status: ValidStatus,
	email_id: String,
}

async fn job_status(
	job_id: i32,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	let rec = sqlx::query_as!(
		JobRecord,
		r#"
		SELECT id, job_uuid, created_at, attempt_at, total_records,
			   total_processed, summary_total_safe, summary_total_invalid,
			   summary_total_risky, summary_total_unknown,
			   job_status as "job_status: _" FROM blk_vrfy_job
		WHERE id = $1
		LIMIT 1
		"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await;

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
