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

use check_if_email_exists::{check_email as ciee_check_email, CheckEmailInput, CheckEmailOutput};
use sentry::protocol::{Event, Value};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, convert::Infallible, env};
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
		"CheckEmailInput".into(),
		Value::String(format!("{:#?}", result)),
	);

	sentry::capture_event(Event {
		extra,
		message: Some(message),
		..Default::default()
	});

	Ok(warp::reply::with_status(
		json,
		StatusCode::INTERNAL_SERVER_ERROR,
	))
}

/// Given an email address (and optionally some additional configuration
/// options), return if email verification details as given by
/// `check_if_email_exists`.
pub async fn check_email(body: EmailInput) -> Result<impl warp::Reply, Infallible> {
	// Create EmailInput for check_if_email_exists from body
	let mut input = CheckEmailInput::new(vec![body.to_email]);
	input
		.from_email(body.from_email.unwrap_or_else(|| "user@example.org".into()))
		.hello_name(body.hello_name.unwrap_or_else(|| "example.org".into()));

	// If relevant ENV vars are set, we proxy.
	match (env::var("RCH_PROXY_HOST"), env::var("RCH_PROXY_PORT")) {
		(Ok(proxy_host), Ok(proxy_port)) => {
			if let Ok(proxy_port) = proxy_port.parse::<u16>() {
				input.proxy(proxy_host, proxy_port);
			}
		}
		_ => (),
	}

	let mut result = ciee_check_email(&input).await;
	let result = result
		.pop()
		.expect("The input has one element, so does the output. qed.");

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
