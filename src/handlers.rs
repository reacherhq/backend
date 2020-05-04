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

use check_if_email_exists::{email_exists, EmailInput as CieeEmailInput, SingleEmail};
use sentry::protocol::{Event, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
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
	result: SingleEmail,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
	let json = warp::reply::json(&result);

	let mut extra = BTreeMap::new();
	extra.insert(
		"SingleEmail".into(),
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
	let mut input = CieeEmailInput::new(body.to_email);
	input
		.from_email(body.from_email.unwrap_or_else(|| "user@example.org".into()))
		.hello_name(body.hello_name.unwrap_or_else(|| "example.org".into()))
		// We proxy through Tor, running locally on 127.0.0.1:9050
		.proxy("127.0.0.1".into(), 9050);

	let result = email_exists(&input).await;

	// We consider `email_exists` failed if:
	// - the email is syntactically correct
	// - AND yet the `mx` or `smtp` field contains an error
	match (&result.syntax, &result.mx, &result.smtp) {
		(Ok(_), Err(error), _) => log_error(format!("{:?}", error), result),
		(Ok(_), _, Err(error)) => log_error(format!("{:?}", error), result),
		_ => Ok(warp::reply::with_status(
			warp::reply::json(&result),
			StatusCode::OK,
		)),
	}
}
