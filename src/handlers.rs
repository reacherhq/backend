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

use async_recursion::async_recursion;
use check_if_email_exists::{check_email as ciee_check_email, CheckEmailInput, CheckEmailOutput};
use sentry::protocol::{Event, Value};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, convert::Infallible, env};
use warp::http::StatusCode;

/// JSON Request from POST /check_email
#[derive(Debug, Deserialize, Serialize)]
pub struct EmailInput {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_email: String,
}

/// Helper function to send an event to Sentry, in case our check_email
/// function fails, and return a 500 error response.
fn log_error(
	message: String,
	result: CheckEmailOutput,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
	let json = warp::reply::json(&result);

	let mut extra = BTreeMap::new();
	extra.insert(
		"CheckEmailOutput".into(),
		Value::String(format!("{:#?}", result)),
	);

	sentry::capture_event(Event {
		extra,
		message: Some(message),
		release: env::var("CARGO_PKG_VERSION").ok().map(Cow::from),
		..Default::default()
	});

	Ok(warp::reply::with_status(
		json,
		StatusCode::INTERNAL_SERVER_ERROR,
	))
}

/// A recursive async function to retry the `ciee_check_email` function
/// multiple times.
///
/// # Panics
///
/// The `input.to_emails` field is assumed to contain exactly email address to
/// check. The function will panic if this field is empty. If it contains more
/// than 1 field, subsequent emails will be ignored.
#[async_recursion]
async fn retry(input: &CheckEmailInput, count: u8) -> CheckEmailOutput {
	let result = ciee_check_email(input)
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");

	// We retry if at least one of the misc, mx or smtp fields contains an
	// error.
	if result.misc.is_ok() && result.mx.is_ok() && result.smtp.is_ok() {
		result
	} else if count <= 1 {
		result
	} else {
		retry(input, count - 1).await
	}
}

/// Given an email address (and optionally some additional configuration
/// options), return if email verification details as given by
/// `check_if_email_exists`.
pub async fn check_email(_: (), body: EmailInput) -> Result<impl warp::Reply, Infallible> {
	// Create EmailInput for check_if_email_exists from body
	let mut input = CheckEmailInput::new(vec![body.to_email]);
	input
		.from_email(body.from_email.unwrap_or_else(|| {
			env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
		}))
		.hello_name(body.hello_name.unwrap_or_else(|| "gmail.com".into()));

	// If relevant ENV vars are set, we proxy.
	if let (Ok(proxy_host), Ok(proxy_port)) =
		(env::var("RCH_PROXY_HOST"), env::var("RCH_PROXY_PORT"))
	{
		if let Ok(proxy_port) = proxy_port.parse::<u16>() {
			input.proxy(proxy_host, proxy_port);
		}
	}

	// Run `ciee_check_email` function 3 times max.
	let result = retry(&input, 3).await;

	// We consider `email_exists` failed if at least one of the misc, mx or smtp
	// fields contains an error.
	match (&result.misc, &result.mx, &result.smtp) {
		(Err(error), _, _) => log_error(format!("{:?}", error), result),
		(_, Err(error), _) => log_error(format!("{:?}", error), result),
		(_, _, Err(error)) => log_error(format!("{:?}", error), result),
		_ => Ok(warp::reply::with_status(
			warp::reply::json(&result),
			StatusCode::OK,
		)),
	}
}
