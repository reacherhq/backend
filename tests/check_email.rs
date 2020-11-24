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

use reacher_backend::routes::{
	check_email::{
		header::{DEFAULT_SAASIFY_SECRET, SAASIFY_SECRET_HEADER},
		post::EndpointRequest,
	},
	create_routes,
};
use serde_json;
use warp::http::StatusCode;
use warp::test::request;

#[tokio::test]
async fn test_missing_header() {
	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes())
		.await;

	println!("{:?}", resp);
	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
	assert_eq!(
		resp.body(),
		r#"Missing request header "x-saasify-proxy-secret""#
	);
}

#[tokio::test]
async fn test_wrong_saasify_secret() {
	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(SAASIFY_SECRET_HEADER, "foo")
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes())
		.await;

	println!("{:?}", resp);
	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
	assert_eq!(
		resp.body(),
		r#"Missing request header "x-saasify-proxy-secret""#
	);
}

#[tokio::test]
async fn test_input_foo_bar() {
	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(SAASIFY_SECRET_HEADER, DEFAULT_SAASIFY_SECRET)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar"}"#).unwrap())
		.reply(&create_routes())
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
		.path("/v0/check_email")
		.method("POST")
		.header(SAASIFY_SECRET_HEADER, DEFAULT_SAASIFY_SECRET)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes())
		.await;

	assert_eq!(resp.status(), StatusCode::OK);
	assert_eq!(
		resp.body(),
		r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#
	);
}
