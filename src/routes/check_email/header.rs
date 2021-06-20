// Reacher - Email Verification
// Copyright (C) 2018-2021 Reacher

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

/// Warp filter to check that the header secret is correct. We accept headers
/// for auth that match:
/// - `x-saasify-proxy-secret`: this means auth is handled by saasify, we don't
/// care about auth anymore.
pub fn check_header() -> warp::filters::BoxedFilter<()> {
	let env_var = env::var("RCH_SAASIFY_SECRET");

	match env_var {
		Ok(saasify_secret) => {
			let saasify_secret: &'static str = Box::leak(Box::new(saasify_secret));

			warp::header::exact("x-saasify-proxy-secret", saasify_secret).boxed()
		}
		Err(_) => warp::any().boxed(),
	}
}
