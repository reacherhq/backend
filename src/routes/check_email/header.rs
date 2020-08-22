// Reacher - Email Verification
// Copyright (C) 2018-2020 Amaury Martiny

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

use super::util::pg_to_warp_error;
use crate::{db::PgPool, errors::ReacherResponseError, models};
use std::{env, str::FromStr};
use uuid::Uuid;
use warp::{http, reject, Filter};

/// The header which holds the Saasify secret.
pub const SAASIFY_SECRET_HEADER: &str = "x-saasify-proxy-secret";

/// The header which holds the Reacher API token.
pub const REACHER_API_TOKEN_HEADER: &str = "x-reacher-api-token";

/// The fallback saasify secret, used in tests and staging.
pub const DEFAULT_SAASIFY_SECRET: &str = "reacher_dev_secret";

/// The secret we retrieve from the headers. It's either the Saasify secret,
/// or a Reacher API token.
#[derive(Debug, PartialEq)]
pub enum HeaderSecret {
	Saasify,
	Reacher(models::api_token::ApiToken),
}

/// Get the server's Saasify secret, either from ENV, or use the fallback dev
/// secret.
fn get_saasify_secret() -> String {
	env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| DEFAULT_SAASIFY_SECRET.into())
}

/// Check that the Reacher API token is correct in the DB.
fn check_api_token(
	pool: PgPool,
	api_token: String,
) -> Result<models::api_token::ApiToken, warp::Rejection> {
	let pool = pool.clone();
	// Get connection from pool.
	let conn = pool.get().map_err(pg_to_warp_error)?;
	// Make sure the api_token in header is a correct UUID.
	let uuid = Uuid::from_str(api_token.as_str()).map_err(|err| {
		reject::custom(ReacherResponseError::new(
			http::StatusCode::BAD_REQUEST,
			format!("Invalid UUID: {}", err),
		))
	})?;
	// Fetch the corresponding ApiToken object from the db.
	let api_token = models::api_token::find_one_by_api_token(&conn, &uuid).map_err(|err| {
		reject::custom(ReacherResponseError::new(
			http::StatusCode::UNAUTHORIZED,
			format!("Cannot find api_token: {}", err),
		))
	})?;

	Ok(api_token)
}

/// Warp filter to check that the header secret is correct. We accept two types
/// of headers for authentication:
/// - x-saasify-proxy-secret: this means auth is handled by saasify, we don't
/// care about saving API usage records in our own DB.
/// - x-reacher-api-token: this means that auth is handled by Reacher, we add
/// an entry in the API usage records.
pub fn check_header(
	pool: PgPool,
) -> impl warp::Filter<Extract = (HeaderSecret,), Error = warp::Rejection> + Clone {
	let saasify_secret = get_saasify_secret();
	// See https://github.com/seanmonstar/warp/issues/503.
	let saasify_secret: &'static str = Box::leak(Box::new(saasify_secret));

	(warp::header::exact_ignore_case(SAASIFY_SECRET_HEADER, saasify_secret)
		.map(|| HeaderSecret::Saasify))
	.or(
		warp::header::<String>(REACHER_API_TOKEN_HEADER).and_then(move |api_token: String| {
			let pool = pool.clone();
			// See https://github.com/seanmonstar/warp/issues/626.
			async move {
				match check_api_token(pool, api_token) {
					Ok(api_token) => Ok(HeaderSecret::Reacher(api_token)),
					Err(err) => Err(err),
				}
			}
		}),
	)
	.unify()
}
