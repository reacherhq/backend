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

#[derive(Debug)]
struct IncorrectSaasifySecret {}

impl warp::reject::Reject for IncorrectSaasifySecret {}

pub const SAASIFY_SECRET_HEADER: &str = "x-saasify-proxy-secret";

/// Get the server's Saasify secret, either from ENV, or use the fallback dev
/// secret.
pub fn get_saasify_secret() -> String {
	env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| "reacher_dev_secret".into())
}

/// Warp filter to check that the Saasify header secret is correct.
pub fn check_saasify_secret() -> impl warp::Filter<Extract = ((),), Error = warp::Rejection> + Clone
{
	warp::header::<String>(SAASIFY_SECRET_HEADER).and_then(|header: String| async move {
		let saasify_secret = get_saasify_secret();

		if header
			.as_bytes()
			.eq_ignore_ascii_case(saasify_secret.as_bytes())
		{
			Ok(())
		} else {
			Err(warp::reject::custom(IncorrectSaasifySecret {}))
		}
	})
}
