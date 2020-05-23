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

use async_recursion::async_recursion;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{
	check_email as ciee_check_email, smtp::SmtpError, CheckEmailInput, CheckEmailOutput,
};
use sentry::protocol::Event;
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
/// function fails.
fn log_error(message: String, result: &CheckEmailOutput, with_proxy: bool) {
	log::debug!("{}", message);

	let mut extra = BTreeMap::new();
	extra.insert("CheckEmailOutput".into(), format!("{:#?}", result).into());
	if let Ok(fly_alloc_id) = env::var("FLY_ALLOC_ID") {
		extra.insert("FLY_ALLOC_ID".into(), fly_alloc_id.into());
	}
	if let Ok(cargo_pkg_version) = env::var("CARGO_PKG_VERSION") {
		extra.insert("CARGO_PKG_VERSION".into(), cargo_pkg_version.into());
	}
	extra.insert("with_proxy".into(), with_proxy.into());

	sentry::capture_event(Event {
		extra,
		message: Some(message),
		// FIXME It seams that this doesn't work on Sentry, so I added it in
		// the `extra` field above too.
		release: env::var("CARGO_PKG_VERSION").ok().map(Cow::from),
		..Default::default()
	});
}

/// A recursive async function to retry the `ciee_check_email` function
/// multiple times, with or without proxy.
///
/// # Panics
///
/// The `input.to_emails` field is assumed to contain exactly email address to
/// check. The function will panic if this field is empty. If it contains more
/// than 1 field, subsequent emails will be ignored.
#[async_recursion]
async fn retry(input: CheckEmailInput, count: u8, with_proxy: bool) -> CheckEmailOutput {
	// We create a local copy of `input`, because we might mutate it for this
	// iteration of the retry process (depending on whether we use Tor or not).
	let mut local_input = input.clone();

	log::debug!(target: "reacher", "Retry #{} for {}", count, local_input.to_emails[0]);

	// If `with_proxy` and relevant ENV vars are set, we proxy.
	if let (true, Ok(proxy_host), Ok(proxy_port)) = (
		with_proxy,
		env::var("RCH_PROXY_HOST"),
		env::var("RCH_PROXY_PORT"),
	) {
		if let Ok(proxy_port) = proxy_port.parse::<u16>() {
			log::debug!(target: "reacher", "Using with_proxy: true");
			// TODO check syntax array
			local_input.proxy(proxy_host, proxy_port);
		}
	}

	let result = ciee_check_email(&local_input)
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");

	// We return the last fetched result, if the retry count is exhausted.
	if count <= 1 {
		result
	} else {
		match (&result.misc, &result.mx, &result.smtp) {
			(Err(error), _, _) => {
				// We log misc errors.
				log_error(format!("{:?}", error), &result, with_proxy);

				// We retry once again.
				retry(input, count - 1, with_proxy).await
			}
			(_, Err(error), _) => {
				// We log mx errors.
				log_error(format!("{:?}", error), &result, with_proxy);

				// We retry once again.
				retry(input, count - 1, with_proxy).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Permanent(response))))
				if (
					// Unable to add <email> because host 23.129.64.184 is listed on zen.spamhaus.org
					// 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
					response.message[0].to_lowercase().contains("spamhaus") ||
					// 5.7.606 Access denied, banned sending IP [23.129.64.216]
					response.message[0].to_lowercase().contains("banned") ||
					// Blocked - see https://ipcheck.proofpoint.com/?ip=23.129.64.192
					// 5.7.1 Mail from 23.129.64.183 has been blocked by Trend Micro Email Reputation Service.
					response.message[0].to_lowercase().contains("blocked") ||
					// 5.7.1 Client host rejected: cannot find your reverse hostname, [23.129.64.184]
					response.message[0].to_lowercase().contains("cannot find your reverse hostname")
				) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// Retry without Tor.
				retry(input, count - 1, false).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
				if (
					// relay not permitted!
					response.message[0].to_lowercase().contains("relay not permitted") ||
					// 23.129.64.216 is not yet authorized to deliver mail from
					response.message[0].to_lowercase().contains("not yet authorized")
				) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// Retry without Tor.
				retry(input, count - 1, false).await
			}
			(_, _, Err(error)) => {
				// If it's a SMTP error we didn't catch above, we log to
				// Sentry, to be able to debug them better. We don't want to
				// spam Sentry and log all instances of the error, hence the
				// `count` check.
				if count <= 3 {
					log_error(format!("{:?}", error), &result, with_proxy);
				}

				// We retry, once with Tor, once without.
				retry(input, count - 1, !with_proxy).await
			}
			// If everything is ok, we just return the result.
			(Ok(_), Ok(_), Ok(_)) => result,
		}
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

	// Run `ciee_check_email` function 4 times max.
	let result = retry(input, 4, true).await;

	Ok(warp::reply::with_status(
		warp::reply::json(&result),
		StatusCode::OK,
	))
}
