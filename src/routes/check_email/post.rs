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

use super::{
	header::{check_header, HeaderSecret},
	known_errors,
	util::{pg_to_warp_error, race_future2},
};
use crate::{db::PgPool, errors::ReacherResponseError, models, sentry_util};
use async_recursion::async_recursion;
use check_if_email_exists::{
	check_email as ciee_check_email, CheckEmailInput, CheckEmailOutput, Reachable,
};
use serde::{Deserialize, Serialize};
use std::{
	convert::Infallible,
	env, fmt,
	time::{Duration, Instant},
};
use warp::{http, reject, Filter};

/// Timeout after which we drop the `check-if-email-exists` check. We run the
/// checks twice (to avoid greylisting), so each verification takes 20s max.
const SMTP_THRESHOLD: u64 = 10;

/// Endpoint request body.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EndpointRequest {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_email: String,
}

/// This option represents how we should execute the SMTP connection to check
/// an email.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryOption {
	/// Use Tor to connect to the SMTP server.
	Tor,
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
enum CheckEmailError {
	/// We get an `is_reachable` Unknown. We consider this internally as an
	/// error case, so that we can do retry mechanisms (see select_ok & retry).
	Unknown((CheckEmailOutput, RetryOption)),
}

/// Converts an endpoint request body into a future that performs email
/// verification.
async fn create_check_email_future(
	body: &EndpointRequest,
	use_tor: bool,
) -> Result<(CheckEmailOutput, RetryOption), CheckEmailError> {
	// FIXME Can we not clone?
	let body = body.clone();

	// Create Request for check_if_email_exists from body
	let mut input = CheckEmailInput::new(vec![body.to_email]);
	input
		.from_email(body.from_email.unwrap_or_else(|| {
			env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
		}))
		.hello_name(body.hello_name.unwrap_or_else(|| "gmail.com".into()));

	// If `use_tor` and relevant ENV vars are set, we proxy.
	if use_tor {
		if let (Ok(proxy_host), Ok(proxy_port)) =
			(env::var("RCH_PROXY_HOST"), env::var("RCH_PROXY_PORT"))
		{
			if let Ok(proxy_port) = proxy_port.parse::<u16>() {
				input.proxy(proxy_host, proxy_port);
			}
		}
	}

	let retry_option = if use_tor {
		RetryOption::Tor
	} else {
		RetryOption::Direct
	};

	input.smtp_timeout(Duration::from_secs(SMTP_THRESHOLD));

	// Retry each future twice, to avoid grey-listing.
	retry(&input, retry_option, 2).await
}

/// Retry the check ciee_check_email function, in particular to avoid
/// greylisting.
#[async_recursion]
async fn retry(
	input: &CheckEmailInput,
	retry_option: RetryOption,
	count: usize,
) -> Result<(CheckEmailOutput, RetryOption), CheckEmailError> {
	log::debug!(
		target:"reacher",
		"[email={}] Checking with retry option {}, attempt #{}",
		input.to_emails[0],
		retry_option,
		count,
	);

	let result = ciee_check_email(&input)
		.await
		.pop()
		.expect("Input contains one email, so does output. qed.");

	log::debug!(
		target:"reacher",
		"[email={}] Got result with retry option {}, attempt #{}, is_reachable={:?}",
		input.to_emails[0],
		retry_option,
		count,
		result.is_reachable
	);

	// If we get an unknown error, log it.
	known_errors::log_unknown_errors(&result, retry_option);

	if result.is_reachable == Reachable::Unknown {
		if count <= 1 {
			Err(CheckEmailError::Unknown((result, retry_option)))
		} else {
			retry(input, retry_option, count - 1).await
		}
	} else {
		Ok((result, retry_option))
	}
}

/// The main `check_email` function that implements the logic of this route.
async fn check_email(
	pool: PgPool,
	header_secret: HeaderSecret,
	body: EndpointRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();

	// Create 2 futures:
	// - one connecting directly to the SMTP server,
	// - the other one connecting to it via Tor.
	// Then race these 2 futures.
	let (value, retry_option) = match race_future2(
		create_check_email_future(&body, false),
		create_check_email_future(&body, true),
	)
	.await
	{
		Ok((value, retry_option)) | Err((CheckEmailError::Unknown((value, retry_option)), _)) => {
			(value, retry_option)
		}
	};

	// Log on Sentry the `is_reachable` field.
	// FIXME We should definitely log this somehwere else than Sentry.
	sentry_util::info(
		format!("is_reachable={:?}", value.is_reachable),
		retry_option,
		now.elapsed().as_millis(),
	);

	// Add a usage record in the db, if the Reacher api token is
	// present.
	if let HeaderSecret::Reacher(api_token) = header_secret {
		// Get connection from pool.
		let conn = pool.get().map_err(pg_to_warp_error)?;

		models::api_usage_record::create_api_usage_record(
			&conn,
			api_token.id,
			"POST",
			"/check_email",
		)
		.map_err(|err| {
			reject::custom(ReacherResponseError::new(
				http::StatusCode::INTERNAL_SERVER_ERROR,
				err.to_string(),
			))
		})?;
	}

	Ok(warp::reply::json(&value))
}

/// Filter to add the DB connection into handlers.
fn with_db_pool(pool: PgPool) -> impl Filter<Extract = (PgPool,), Error = Infallible> + Clone {
	warp::any().map(move || pool.clone())
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email(
	pool: PgPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::path!("v0" / "check_email")
		.and(warp::post())
		.and(with_db_pool(pool.clone()))
		.and(check_header(pool))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(check_email)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
