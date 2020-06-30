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

use reacher_backend::{
	check_email_heroku,
	saasify_secret::{get_saasify_secret, IncorrectSaasifySecret, SAASIFY_SECRET_HEADER},
	sentry_util::CARGO_PKG_VERSION,
	setup_sentry, ReacherInput,
};
use std::{convert::Infallible, env, net::IpAddr};
use warp::http::StatusCode;
use warp::Filter;

/// Warp filter to check that the Saasify header secret is correct.
fn check_saasify_secret() -> impl warp::Filter<Extract = ((),), Error = warp::Rejection> + Clone {
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

/// Given an email address (and optionally some additional configuration
/// options), return if email verification details as given by
/// `check_if_email_exists`.
async fn check_email(_: (), body: ReacherInput) -> Result<impl warp::Reply, Infallible> {
	let result = check_email_heroku(body).await;

	Ok(warp::reply::with_status(
		warp::reply::json(&result),
		StatusCode::OK,
	))
}

/// Create all the endpoints of our API.
fn create_api() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	// POST /check_email
	let post_check_email = warp::path("check_email")
		.and(warp::post())
		// FIXME We should be able to just use warp::header::exact, and remove
		// completely `./saasify_secret.rs`.
		// https://github.com/seanmonstar/warp/issues/503
		.and(check_saasify_secret())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(check_email)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"));

	// GET /version
	// This is mainly used for Heroku keep alive.
	let get_version = warp::path("version")
		.and(warp::get())
		.map(|| CARGO_PKG_VERSION);

	get_version.or(post_check_email)
}

/// Run a HTTP server using warp.
///
/// # Panics
///
/// If at least one of the environment variables:
/// - RCH_HTTP_HOST
/// - RCH_PROXY_HOST
/// - RCH_PROXY_PORT
/// is malformed, then the program will panic.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();
	let _guard = setup_sentry();

	let api = create_api();

	let host = env::var("RCH_HTTP_HOST")
		.unwrap_or_else(|_| "127.0.0.1".into())
		.parse::<IpAddr>()
		.expect("Env var RCH_HTTP_HOST is malformed.");
	let port = env::var("PORT")
		.map(|port| port.parse::<u16>().expect("Env var PORT is malformed."))
		.unwrap_or(8080);
	log::info!(target: "reacher", "Server is listening on {}:{}.", host, port);

	warp::serve(api).run((host, port)).await;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::create_api;
	use reacher_backend::{saasify_secret::SAASIFY_SECRET_HEADER, ReacherInput};
	use serde_json;
	use warp::http::StatusCode;
	use warp::test::request;

	#[tokio::test]
	async fn test_missing_saasify_secret() {
		let resp = request()
			.path("/check_email")
			.method("POST")
			.reply(&create_api())
			.await;

		assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
		assert_eq!(
			resp.body(),
			"Missing request header \"x-saasify-proxy-secret\"".as_bytes()
		);
	}

	#[tokio::test]
	async fn test_incorrect_saasify_secret() {
		let resp = request()
			.path("/check_email")
			.method("POST")
			.header(SAASIFY_SECRET_HEADER, "incorrect")
			.reply(&create_api())
			.await;

		assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(
			resp.body(),
			"Unhandled rejection: IncorrectSaasifySecret".as_bytes()
		);
	}

	#[tokio::test]
	async fn test_input_foo_bar() {
		let resp = request()
			.path("/check_email")
			.method("POST")
			.header(SAASIFY_SECRET_HEADER, "reacher_dev_secret")
			.json(&serde_json::from_str::<ReacherInput>(r#"{"to_email": "foo@bar"}"#).unwrap())
			.reply(&create_api())
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
			.header(SAASIFY_SECRET_HEADER, "reacher_dev_secret")
			.json(&serde_json::from_str::<ReacherInput>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
			.reply(&create_api())
			.await;

		assert_eq!(resp.status(), StatusCode::OK);
		assert_eq!(
			resp.body(),
			r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#
		);
	}
}
