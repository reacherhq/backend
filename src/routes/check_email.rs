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

//! This file implements the `POST /check_email` endpoint.

use crate::sentry_util;
use check_if_email_exists::{check_email as ciee_check_email, CheckEmailInput};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, env, fmt, time::Instant};
use warp::Filter;

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct EndpointRequest {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_email: String,
}

impl Into<CheckEmailInput> for EndpointRequest {
	fn into(self) -> CheckEmailInput {
		// Create Request for check_if_email_exists from body
		let mut input = CheckEmailInput::new(vec![self.to_email]);
		input
			.from_email(self.from_email.unwrap_or_else(|| {
				env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
			}))
			.hello_name(self.hello_name.unwrap_or_else(|| "gmail.com".into()));

		input
	}
}

/// This option represents how we should execute the SMTP connection to check
/// an email.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryOption {
	/// Use Tor to connect to the SMTP server.
	Tor,
	/// Heroku connects to the SMTP server directly.
	Heroku,
}

impl fmt::Display for RetryOption {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

/// The main `check_email` function that implements the logic of this route.
async fn check_email(body: EndpointRequest) -> Result<impl warp::Reply, Infallible> {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();
	let result = ciee_check_email(&body.into())
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");

	// Log on Sentry
	sentry_util::info(
		format!("is_reachable={:?}", result.is_reachable),
		RetryOption::Heroku,
		now.elapsed().as_millis(),
	);

	Ok(warp::reply::json(&result))
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
	warp::path("check_email")
		.and(warp::post())
		// TODO ADD AUTH
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(check_email)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}

#[cfg(test)]
mod tests {
	use super::{post_check_email, EndpointRequest};
	use serde_json;
	use warp::http::StatusCode;
	use warp::test::request;

	#[tokio::test]
	async fn test_input_foo_bar() {
		let resp = request()
			.path("/check_email")
			.method("POST")
			.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar"}"#).unwrap())
			.reply(&post_check_email())
			.await;

		assert_eq!(resp.status(), StatusCode::OK);
		assert_eq!(
			resp.body(),
			r#"{"input":"foo@bar","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":null,"domain":"","is_valid_syntax":false,"username":""}}"#
		);
	}

	#[tokio::test]
	async fn test_input_foo_bar_baz() {
		let resp = request()
			.path("/check_email")
			.method("POST")
			.json(
				&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap(),
			)
			.reply(&post_check_email())
			.await;

		assert_eq!(resp.status(), StatusCode::OK);
		assert_eq!(
			resp.body(),
			r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#
		);
	}
}
