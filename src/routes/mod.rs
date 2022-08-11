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

#[cfg(feature = "bulk")]
pub mod bulk;
pub mod check_email;
pub mod version;

use super::errors;
#[cfg(feature = "bulk")]
use sqlx::{Pool, Postgres};
use warp::Filter;

#[cfg(feature = "bulk")]
pub fn create_routes(
	conn_pool: Pool<Postgres>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(check_email::post::post_check_email())
		.or(bulk::post::create_bulk_job(conn_pool.clone()))
		.or(bulk::get::get_bulk_job_status(conn_pool.clone()))
		.or(bulk::download::get_bulk_job_result(conn_pool))
		.recover(errors::handle_rejection)
}

#[cfg(not(feature = "bulk"))]
pub fn create_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	version::get::get_version()
		.or(check_email::post::post_check_email())
		.recover(errors::handle_rejection)
}
