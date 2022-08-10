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

//! This file implements the `GET /bulk/{id}` endpoint.

use crate::errors::ReacherError;
use serde::Serialize;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use warp::Filter;

/// NOTE: Type conversions from postgres to rust types
/// are according to the table given by
/// [sqlx here](https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html)
#[derive(Debug, Serialize, PartialEq, Eq)]
enum ValidStatus {
	Running,
	Completed,
}

/// Job record stores the information about a submitted job
///
/// `job_status` field is an update on read field. It's
/// status will be derived from counting number of
/// completed email verification tasks. It will be updated
/// with the most recent status of the job.
#[derive(sqlx::FromRow, Debug, Serialize)]
struct JobRecord {
	id: i32,
	created_at: DateTime<Utc>,
	total_records: i32,
}

/// Summary of a bulk verification job status
#[derive(Debug, Serialize)]
struct JobStatusSummaryResponseBody {
	total_safe: i32,
	total_risky: i32,
	total_invalid: i32,
	total_unknown: i32,
}

/// Complete information about a bulk verification job
#[derive(Debug, Serialize)]
struct JobStatusResponseBody {
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
	let job_rec = sqlx::query_as!(
		JobRecord,
		r#"
		SELECT id, created_at, total_records FROM bulk_jobs
		WHERE id = $1
		LIMIT 1
		"#,
		job_id
	)
	.fetch_one(&conn_pool)
	.await
	.map_err(|e| {
		log::error!(
			target:"reacher",
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
		FROM email_results
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

	let job_status = if (agg_info.total_processed.unwrap() as i32) < job_rec.total_records {
		ValidStatus::Running
	} else {
		ValidStatus::Completed
	};

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
		job_status,
	}))
}

pub fn get_bulk_job_status(
	conn_pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::path!("v0" / "bulk" / i32)
		.and(warp::get())
		.and_then(move |job_id| job_status(job_id, conn_pool.clone()))
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
