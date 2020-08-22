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

#[macro_use]
extern crate diesel_migrations;

#[macro_use]
mod common;

use common::{create_test_user, teardown_pool};
use reacher_backend::{
	db::{connect_db, PgPool},
	models::api_usage_record::get_api_usage_records_by_api_token,
	routes::{
		check_email::{
			header::{DEFAULT_SAASIFY_SECRET, REACHER_API_TOKEN_HEADER, SAASIFY_SECRET_HEADER},
			post::EndpointRequest,
		},
		create_routes,
	},
};
use serde_json;
use std::env;
use std::sync::Once;
use warp::http::StatusCode;
use warp::test::request;

embed_migrations!();

// Run some stuff once.
// https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust
static INIT: Once = Once::new();

/// Create a database pool after running all migrations. FIXME this file should
/// go inside common.rs.
pub fn setup_pool() -> PgPool {
	let database_url =
		env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/reacher".into());

	let pool = connect_db(&database_url);
	let connection = pool
		.get()
		.expect("DB pool is expected to be defined in tests. qed.");

	INIT.call_once(|| {
		embedded_migrations::run(&connection).expect("Migrations should pass. qed.");
	});

	pool
}

#[tokio::test]
async fn test_missing_header() {
	let pool = setup_pool();

	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool))
		.await;

	println!("{:?}", resp);
	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
	assert_eq!(
		resp.body(),
		r#"Missing request header "x-reacher-api-token""#
	);
}

#[tokio::test]
async fn test_wrong_saasify_secret() {
	let pool = setup_pool();

	let resp = request()
		.path("/check_email")
		.method("POST")
		.header(SAASIFY_SECRET_HEADER, "foo")
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool))
		.await;

	println!("{:?}", resp);
	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
	assert_eq!(
		resp.body(),
		r#"Missing request header "x-reacher-api-token""#
	);
}

#[tokio::test]
async fn test_api_token_not_uuid() {
	let pool = setup_pool();

	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(REACHER_API_TOKEN_HEADER, "foo")
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool))
		.await;

	assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
	assert_eq!(
		resp.body(),
		r#"{"message":"Invalid UUID: invalid length: expected one of [36, 32], found 3"}"#
	);
}

#[tokio::test]
async fn test_api_token_not_in_db() {
	let pool = setup_pool();
	let (alice, _) = create_test_user(&pool);

	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(
			REACHER_API_TOKEN_HEADER,
			"a87429e2-d46d-45d4-b184-c4583350f7f3",
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
	assert_eq!(
		resp.body(),
		r#"{"message":"Cannot find api_token: NotFound"}"#
	);

	teardown_pool(pool, vec![&alice.id]);
}

#[tokio::test]
async fn test_input_foo_bar() {
	let pool = setup_pool();
	let (alice, alice_api_token) = create_test_user(&pool);

	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(
			REACHER_API_TOKEN_HEADER,
			alice_api_token.api_token.to_string(),
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	assert_eq!(resp.status(), StatusCode::OK);
	assert_eq!(
		resp.body(),
		r#"{"input":"foo@bar","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":null,"domain":"","is_valid_syntax":false,"username":""}}"#
	);

	teardown_pool(pool, vec![&alice.id]);
}

#[tokio::test]
async fn test_input_foo_bar_baz() {
	let pool = setup_pool();
	let (alice, alice_api_token) = create_test_user(&pool);

	let resp = request()
		.path("/v0/check_email")
		.method("POST")
		.header(
			REACHER_API_TOKEN_HEADER,
			alice_api_token.api_token.to_string(),
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	assert_eq!(resp.status(), StatusCode::OK);
	assert_eq!(
		resp.body(),
		r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#
	);

	teardown_pool(pool, vec![&alice.id]);
}
#[tokio::test]
async fn test_input_foo_bar_baz_with_saasify_secret() {
	let pool = setup_pool();

	let resp = request()
		.path("/check_email")
		.method("POST")
		.header(SAASIFY_SECRET_HEADER, DEFAULT_SAASIFY_SECRET)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	assert_eq!(resp.status(), StatusCode::OK);
	assert_eq!(
		resp.body(),
		r#"{"input":"foo@bar.baz","is_reachable":"invalid","misc":{"is_disposable":false,"is_role_account":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}"#
	);
}

#[tokio::test]
async fn test_api_usage_record() {
	let pool = setup_pool();
	let (alice, alice_api_token) = create_test_user(&pool);
	let connection = pool
		.get()
		.expect("DB pool is expected to be defined in tests. qed.");

	// Send 2 requests.
	let _ = request()
		.path("/v0/check_email")
		.method("POST")
		.header(
			REACHER_API_TOKEN_HEADER,
			alice_api_token.api_token.to_string(),
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;
	let _ = request()
		.path("/v0/check_email")
		.method("POST")
		.header(
			REACHER_API_TOKEN_HEADER,
			alice_api_token.api_token.to_string(),
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	let records = get_api_usage_records_by_api_token(&connection, alice_api_token.id)
		.expect("Getting api usage records should work. qed.");
	assert_eq!(records.len(), 2);

	teardown_pool(pool, vec![&alice.id]);
}

#[tokio::test]
async fn test_no_api_usage_record_with_saasify() {
	let pool = setup_pool();
	let (alice, alice_api_token) = create_test_user(&pool);
	let connection = pool
		.get()
		.expect("DB pool is expected to be defined in tests. qed.");

	// Send 2 requests.
	let _ = request()
		.path("/check_email")
		.method("POST")
		// We put both headers. Only the Saasify one should be taken into
		// account.
		.header(SAASIFY_SECRET_HEADER, DEFAULT_SAASIFY_SECRET)
		.header(
			REACHER_API_TOKEN_HEADER,
			alice_api_token.api_token.to_string(),
		)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;
	let _ = request()
		.path("/check_email")
		.method("POST")
		// We only put the Saasify header here.
		.header(SAASIFY_SECRET_HEADER, DEFAULT_SAASIFY_SECRET)
		.json(&serde_json::from_str::<EndpointRequest>(r#"{"to_email": "foo@bar.baz"}"#).unwrap())
		.reply(&create_routes(pool.clone()))
		.await;

	let records = get_api_usage_records_by_api_token(&connection, alice_api_token.id)
		.expect("Getting api usage records should work. qed.");
	assert_eq!(records.len(), 0);

	teardown_pool(pool, vec![&alice.id]);
}
