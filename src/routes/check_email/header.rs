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

use std::env;
use warp::Filter;

/// The header which holds the Saasify secret.
pub const SAASIFY_SECRET_HEADER: &str = "x-saasify-proxy-secret";

/// The fallback saasify secret, used in tests and staging.
pub const DEFAULT_SAASIFY_SECRET: &str = "reacher_dev_secret";

/// The secret we retrieve from the headers. For now it's a Saasify secret,
/// but there might be others in the future
#[derive(Debug, PartialEq)]
pub enum HeaderSecret {
	Saasify,
}

/// Get the server's Saasify secret, either from ENV, or use the fallback dev
/// secret.
fn get_saasify_secret() -> String {
	env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| DEFAULT_SAASIFY_SECRET.into())
}

/// Warp filter to check that the header secret is correct. We accept headers
/// for auth that match:
/// - `x-saasify-proxy-secret`: this means auth is handled by saasify, we don't
/// care about auth anymore.
/// - `Authorization`: this is a temporary fix to allow all requests with this
/// header.
pub fn check_header(
) -> impl warp::Filter<Extract = (HeaderSecret,), Error = warp::Rejection> + Clone {
	let saasify_secret = get_saasify_secret();
	// See https://github.com/seanmonstar/warp/issues/503.
	let saasify_secret: &'static str = Box::leak(Box::new(saasify_secret));

	warp::header::exact_ignore_case(SAASIFY_SECRET_HEADER, saasify_secret)
		.map(|| HeaderSecret::Saasify)
}
