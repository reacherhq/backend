// Reacher
// Copyright (C) 2018-2020 Amaury Martiny

// Reacher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Reacher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Reacher.  If not, see <http://www.gnu.org/licenses/>.

use std::env;
use warp::Filter;

#[derive(Debug)]
struct IncorrectSaasifySecret {}

impl warp::reject::Reject for IncorrectSaasifySecret {}

pub const SAASIFY_SECRET_HEADER: &str = "x-saasify-proxy-secret";

pub fn check_saasify_secret() -> impl warp::Filter<Extract = ((),), Error = warp::Rejection> + Clone
{
	warp::header::<String>(SAASIFY_SECRET_HEADER).and_then(|header: String| async move {
		let saasify_secret =
			env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| "reacher_dev_secret".into());

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
