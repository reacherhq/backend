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

use super::check::check_email_serverless;
use crate::{
	saasify_secret::{get_saasify_secret, IncorrectSaasifySecret, SAASIFY_SECRET_HEADER},
	setup_sentry, ReacherInput, ReacherOutputError,
};
use lambda_http::{
	ext::PayloadError,
	http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, ORIGIN},
	IntoResponse, Request, Response,
};
use std::fmt;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

/// Input errors on the endpoint.
#[derive(Debug)]
enum CheckEmailInputError {
	/// Request payload deserialization errors.
	PayloadError(PayloadError),
	/// Error with missing or incorrect `x-saasify-secret` header.
	IncorrectSaasifySecret,
}

impl From<serde_json::Error> for CheckEmailInputError {
	fn from(err: serde_json::Error) -> Self {
		CheckEmailInputError::PayloadError(PayloadError::Json(err))
	}
}

impl fmt::Display for CheckEmailInputError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CheckEmailInputError::PayloadError(err) => writeln!(f, "{}", err),
			CheckEmailInputError::IncorrectSaasifySecret => {
				writeln!(f, "{}", IncorrectSaasifySecret::new())
			}
		}
	}
}

/// Make sure the input of the request is well-formed.
fn sanitize_input(request: Request) -> Result<ReacherInput, CheckEmailInputError> {
	let saasify_secret = get_saasify_secret();

	let saasify_input = match request.headers().get(SAASIFY_SECRET_HEADER) {
		Some(x) => x,
		None => {
			return Err(CheckEmailInputError::IncorrectSaasifySecret);
		}
	};

	if !saasify_input
		.as_bytes()
		.eq_ignore_ascii_case(saasify_secret.as_bytes())
	{
		return Err(CheckEmailInputError::IncorrectSaasifySecret);
	}

	let result = serde_json::from_slice::<ReacherInput>(request.body())?;

	Ok(result)
}

/// Handler around `check_email_serverless` to integrate with lambda.
pub async fn lambda_check_email_handler(request: Request) -> Result<impl IntoResponse, Error> {
	let _guard = setup_sentry();

	let mut response = Response::builder();

	// If request has origin, send back an access allow origin.
	if let Some(origin) = request.headers().get(ORIGIN) {
		response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, origin);
	}

	let input = match sanitize_input(request) {
		Ok(x) => x,
		Err(err) => {
			let err = ReacherOutputError::new(err);

			return Ok(response
				.status(400)
				.body(serde_json::to_string(&err).expect("`err` is serializable. qed."))
				.expect("Correct response body. qed."));
		}
	};

	let result = check_email_serverless(input).await;

	Ok(response
		.status(200)
		.header(CONTENT_TYPE, "application/json")
		.body(serde_json::to_string(&result).expect("`result` is serializable. qed."))
		.expect("`result` is serializable. qed."))
}

#[cfg(test)]
mod tests {
	use super::*;
	use lambda_http::Request;
	use serde_json::json;

	#[tokio::test]
	async fn test_missing_saasify_secret() {
		let request = Request::new(r#"{"to_email": "foo@bar.baz"}"#.into());

		let expected = json!({
			"error": "IncorrectSaasifySecret\n"
		})
		.into_response();

		let response = lambda_check_email_handler(request)
			.await
			.unwrap()
			.into_response();
		assert_eq!(response.body(), expected.body())
	}

	#[tokio::test]
	async fn test_incorrect_saasify_secret() {
		let mut request = Request::new(r#"{"to_email": "foo@bar.baz"}"#.into());
		let headers = request.headers_mut();
		headers.insert("SAASIFY_SECRET_HEADER", "incorrect".parse().unwrap());

		let expected = json!({
			"error": "IncorrectSaasifySecret\n"
		})
		.into_response();

		let response = lambda_check_email_handler(request.into())
			.await
			.unwrap()
			.into_response();
		assert_eq!(response.body(), expected.body())
	}

	#[tokio::test]
	async fn test_input_foo_bar() {
		let mut request = Request::new(r#"{"to_email": "foo@bar"}"#.into());
		let headers = request.headers_mut();
		headers.insert(SAASIFY_SECRET_HEADER, "reacher_dev_secret".parse().unwrap());

		let expected = r#"{"input":"foo@bar","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":null,"domain":"","is_valid_syntax":false,"username":""}}"#.into_response();
		let response = lambda_check_email_handler(request.into())
			.await
			.unwrap()
			.into_response();

		assert_eq!(response.body(), expected.body())
	}

	#[tokio::test]
	async fn test_input_foo_bar_baz() {
		let mut request = Request::new(r#"{"to_email": "foo@bar.baz"}"#.into());
		let headers = request.headers_mut();
		headers.insert(SAASIFY_SECRET_HEADER, "reacher_dev_secret".parse().unwrap());

		let expected = r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#.into_response();
		let response = lambda_check_email_handler(request.into())
			.await
			.unwrap()
			.into_response();

		assert_eq!(response.body(), expected.body())
	}
}
