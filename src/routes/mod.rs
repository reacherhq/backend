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

pub mod bulk;
mod check_email;
mod version;

use super::errors;
use sqlx::{Pool, Postgres};
use warp::{Filter, Rejection};

pub fn create_routes(
	o: Option<Pool<Postgres>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
	let is_enabled = o.is_some();

	// Conditional routes that are added only if the conn_pool_option is not
	// empty.
	let post = with_bulk(is_enabled).and(bulk::post::create_bulk_job(o.unwrap().clone()));
	let get = with_bulk(is_enabled).and(bulk::get::get_bulk_job_status(o.unwrap().clone()));
	let results = with_bulk(is_enabled).and(bulk::results::get_bulk_job_result(o.unwrap()));

	version::get::get_version()
		.or(check_email::post::post_check_email())
		.or(post)
		.or(get)
		.or(results)
		.recover(errors::handle_rejection)
}

fn with_bulk(is_bulk_enabled: bool) -> impl Filter<Extract = ((),), Error = Rejection> + Copy {
	warp::any().and_then(async move || {
		if is_bulk_enabled {
			Ok(())
		} else {
			Err(warp::reject::not_found())
		}
	})
}
