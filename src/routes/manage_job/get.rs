use crate::errors::ReacherError;
use crate::routes::check_email::header::check_header;
use sqlx::{Pool, Postgres};
use warp::Filter;

use serde::Serialize;

use check_if_email_exists::CheckEmailOutput;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::Json;

#[derive(sqlx::Type, Debug, Serialize, PartialEq, Eq)]
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

/// Summary of a bulk verification job status
#[derive(Debug, Serialize)]
pub struct JobStatusSummaryResponseBody {
	total_safe: i32,
	total_risky: i32,
	total_invalid: i32,
	total_unknown: i32,
}

/// Complete information about a bulk verification job
#[derive(Debug, Serialize)]
pub struct JobStatusResponseBody {
	job_id: i32,
	created_at: DateTime<Utc>,
	total_records: i32,
	total_processed: i32,
	summary: JobStatusSummaryResponseBody,
	job_status: ValidStatus,
}

async fn job_status(
	job_id: i32,
	conn_pool: Pool<Postgres>,
) -> Result<impl warp::Reply, warp::Rejection> {
	let mut job_rec = sqlx::query_as!(
		JobRecord,
		r#"
		SELECT id, created_at, total_records, job_status as "job_status: _" FROM blk_vrfy_job
		WHERE id = $1
		LIMIT 1
		"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target:"reacher/v0/bulk/",
			"Failed to get job record for [job_id={}] with [error={}]",
			job_id,
			e
		);
		ReacherError::from(e)
	})?;

	let agg_info = sqlx::query!(
		r#"
		SELECT
			COUNT(*) as total_processed,
			COUNT(CASE WHEN result ->> 'is_reachable' LIKE 'safe' THEN 1 END) as safe_count,
			COUNT(CASE WHEN result ->> 'is_reachable' LIKE 'risky' THEN 1 END) as risky_count,
			COUNT(CASE WHEN result ->> 'is_reachable' LIKE 'invalid' THEN 1 END) as invalid_count,
			COUNT(CASE WHEN result ->> 'is_reachable' LIKE 'unknown' THEN 1 END) as unknown_count
		FROM ema_vrfy_rec
		WHERE job_id = $1
		"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target:"reacher/v0/bulk/",
			"Failed to get aggregate info for [job_id={}] with [error={}]",
			job_id,
			e
		);
		ReacherError::from(e)
	})?;

	if job_rec.job_status == ValidStatus::Running {
		if job_rec.total_records == (agg_info.total_processed.unwrap() as i32) {
			// update job status to completed
			sqlx::query_as!(
				JobRecord,
				r#"
				UPDATE blk_vrfy_job
				SET job_status = $1
				WHERE id = $2
				"#,
				ValidStatus::Completed as ValidStatus,
				job_id
			)
			.fetch_one(&conn_pool)
			.await
			.map_or_else(
				|e| {
					log::error!(
						target:"reacher/v0/bulk/",
						"Failed to update job status to completed for [job_id={}] with [error={}]",
						job_id,
						e
					);
				},
				|_| {
					log::info!(
						target:"reacher/v0/bulk/",
						"Update job status to completed for [job_id={}]",
						job_id,
					);

					job_rec.job_status = ValidStatus::Completed;
				},
			);
		}
	}

	Ok(warp::reply::json(&JobStatusResponseBody {
		job_id: job_rec.id,
		created_at: job_rec.created_at,
		total_records: job_rec.total_records,
		total_processed: agg_info.total_processed.unwrap() as i32,
		summary: JobStatusSummaryResponseBody {
			total_safe: agg_info.safe_count.unwrap() as i32,
			total_risky: agg_info.risky_count.unwrap() as i32,
			total_invalid: agg_info.invalid_count.unwrap() as i32,
			total_unknown: agg_info.unknown_count.unwrap() as i32,
		},
		job_status: job_rec.job_status,
	}))
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
