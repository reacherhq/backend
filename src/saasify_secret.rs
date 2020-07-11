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

//! Helper functions to handle Saasify's secret.

use std::{env, fmt};

#[derive(Debug)]
pub struct IncorrectSaasifySecret {}

impl Default for IncorrectSaasifySecret {
	fn default() -> Self {
		IncorrectSaasifySecret {}
	}
}

impl IncorrectSaasifySecret {
	pub fn new() -> Self {
		Default::default()
	}
}

impl fmt::Display for IncorrectSaasifySecret {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl warp::reject::Reject for IncorrectSaasifySecret {}

pub const SAASIFY_SECRET_HEADER: &str = "x-saasify-proxy-secret";

/// Get the server's Saasify secret, either from ENV, or use the fallback dev
/// secret.
pub fn get_saasify_secret() -> String {
	env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| "reacher_dev_secret".into())
}
