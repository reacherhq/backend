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

use super::{header::check_header, known_errors};
use crate::sentry_util;
use async_recursion::async_recursion;
use check_if_email_exists::{
	check_email as ciee_check_email, CheckEmailInput, CheckEmailInputProxy, CheckEmailOutput,
	Reachable,
};
use serde::{Deserialize, Serialize};
use std::{
	env, fmt,
	time::{Duration, Instant},
};
use warp::Filter;

/// Timeout after which we drop the `check-if-email-exists` check. We run the
/// checks twice (to avoid greylisting), so each verification takes 20s max.
const SMTP_THRESHOLD: u64 = 10;

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

/// Converts an endpoint request body into a future that performs email
/// verification.
async fn create_check_email_future(
	body: &EndpointRequest,
) -> Result<(CheckEmailOutput, RetryOption), CheckEmailError> {
	// FIXME Can we not clone?
	let body = body.clone();

	// Create Request for check_if_email_exists from body
	let mut input = CheckEmailInput::new(vec![body.to_email]);
	input
		.set_from_email(body.from_email.unwrap_or_else(|| {
			env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
		}))
		.set_hello_name(body.hello_name.unwrap_or_else(|| "gmail.com".into()));

	if let Some(proxy_input) = body.proxy {
		input.set_proxy(proxy_input);
	}

	if let Some(smtp_port) = body.smtp_port {
		input.set_smtp_port(smtp_port);
	}

	input.set_smtp_timeout(Duration::from_secs(SMTP_THRESHOLD));

	// Retry each future twice, to avoid grey-listing.
	retry(&input, RetryOption::Direct, 2).await
}

/// Retry the check ciee_check_email function, in particular to avoid
/// greylisting.
/// NOTE: This function currently expects only 1 input per task
/// if the task size in `EMAIL_TASK_BATCH_SIZE` is made greater than 1
/// this will have to change to handle a list of inputs
#[async_recursion]
pub async fn retry(
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

	let result = ciee_check_email(input)
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
async fn check_email(body: EndpointRequest) -> Result<impl warp::Reply, warp::Rejection> {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();

	// Run the future to check an email.
	let (value, retry_option) = match create_check_email_future(&body).await {
		Ok((value, retry_option)) | Err(CheckEmailError::Unknown((value, retry_option))) => {
			(value, retry_option)
		}
	};

	// Log on Sentry the `is_reachable` field.
	// We should definitely log this somewhere else than Sentry.
	// TODO https://github.com/reacherhq/backend/issues/207
	sentry_util::metrics(
		format!("is_reachable={:?}", value.is_reachable),
		retry_option,
		now.elapsed().as_millis(),
		value.syntax.domain.as_ref(),
	);

	Ok(warp::reply::json(&value))
}

/// Create the `POST /check_email` endpoint.
pub fn post_check_email() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
	warp::path!("v0" / "check_email")
		.and(warp::post())
		.and(check_header())
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(check_email)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"))
}
