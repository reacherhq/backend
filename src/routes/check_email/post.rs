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

//! This file implements the `POST /check_email` endpoint.

use crate::check::check_email;
use check_if_email_exists::{CheckEmailInput, CheckEmailInputProxy, CheckEmailOutput};
use serde::{Deserialize, Serialize};
use std::{env, fmt};
use warp::Filter;

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndpointRequest {
	from_email: Option<String>,
	hello_name: Option<String>,
	proxy: Option<CheckEmailInputProxy>,
	smtp_port: Option<u16>,
	to_email: String,
}

/// This option represents how we should execute the SMTP connection to check
/// an email.
/// For now, we only support directly connecting to the SMTP server, but in the
/// future, we might try proxying.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryOption {
	/// Heroku connects to the SMTP server directly.
	Direct,
}

impl fmt::Display for RetryOption {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

/// Errors that can happen during an email verification.
#[derive(Debug)]
pub enum CheckEmailError {
	/// We get an `is_reachable` Unknown. We consider this internally as an
	/// error case, so that we can do retry mechanisms (see select_ok & retry).
	Unknown((CheckEmailOutput, RetryOption)),
}

impl Into<CheckEmailInput> for EndpointRequest {
	fn into(self) -> CheckEmailInput {
		// Create Request for check_if_email_exists from body
		let mut input = CheckEmailInput::new(vec![self.to_email]);
		input
			.set_from_email(self.from_email.unwrap_or_else(|| {
				env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
			}))
			.set_hello_name(self.hello_name.unwrap_or_else(|| "gmail.com".into()));

		if let Some(proxy_input) = self.proxy {
			input.set_proxy(proxy_input);
		}

		if let Some(smtp_port) = self.smtp_port {
			input.set_smtp_port(smtp_port);
		}

		input
	}
}

/// The main endpoint handler that implements the logic of this route.
async fn handler(body: EndpointRequest) -> Result<impl warp::Reply, warp::Rejection> {
	// Run the future to check an email.
	Ok(warp::reply::json(&check_email(&body.into()).await))
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
	warp::path!("v0" / "check_email")
		.and(warp::post())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handler)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
