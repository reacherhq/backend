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

use super::known_errors::{
	has_smtp_io_errors, has_smtp_permanent_errors, has_smtp_transient_errors,
};
use crate::{
	saasify_secret::get_saasify_secret, sentry_util, ReacherInput, ReacherOutput, RetryOption,
};
use async_recursion::async_recursion;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{check_email as ciee_check_email, smtp::SmtpError, CheckEmailInput};
use serde_json::Value;
use std::{env, time::Instant};

/// A recursive async function to retry the `ciee_check_email` function
/// multiple times, depending on the environment we're in. Returns a Tuple,
/// where the first element is the result, and subsequent elements are the
/// options passed into the
///
/// # Panics
///
/// The `input.to_emails` field is assumed to contain exactly email address to
/// check. The function will panic if this field is empty. If it contains more
/// than 1 field, subsequent emails will be ignored.
#[async_recursion]
async fn retry(body: ReacherInput, count: u8, option: RetryOption) -> (ReacherOutput, RetryOption) {
	log::info!(target: "reacher", "Retry #{}, with proxy {:?}", count, option);

	// If we're using Heroku option, then we make a HTTP call to Heroku.
	if option == RetryOption::Heroku {
		return match reqwest::Client::new()
			.post("https://reacher-us-1.herokuapp.com/check_email")
			.header("Content-Type", "application/json")
			.header("x-saasify-proxy-secret", get_saasify_secret())
			.json(&body)
			.send()
			.await
		{
			Ok(response) => {
				// Parse Heroku's result, to see what `is_reachable` we got.
				match response.json().await {
					Ok(result) => {
						// Parse Heroku's result, to see what `is_reachable` we got.
						let result: Value = result; // FIXME Why is this line needed?
						let is_reachable = result
							.as_object()
							.and_then(|obj| obj.get("is_reachable"))
							.and_then(|is_reachable| is_reachable.as_str());

						match is_reachable {
							Some(is_reachable) => {
								// If Heroku also returns "unknown", then we retry.
								if is_reachable == "unknown" {
									retry(body, count - 1, RetryOption::Tor).await
								} else {
									(ReacherOutput::Json(result), option)
								}
							}
							// If somehow we couldn't parse the Heroku response, we retry.
							None => {
								sentry_util::error(
									"Heroku cannot parse response".to_string(),
									Some(format!("{:#?}", result).as_ref()),
									RetryOption::Heroku,
								);
								retry(body, count - 1, RetryOption::Tor).await
							}
						}
					}
					Err(err) => {
						sentry_util::error(
							format!("Cannot deserialize Heroku response: {}", err.to_string()),
							Some(format!("{:#?}", err).as_ref()),
							option,
						);

						retry(body, count - 1, RetryOption::Tor).await
					}
				}
			}
			Err(err) => {
				sentry_util::error(
					format!("Heroku not returning 200: {}", err.to_string()),
					Some(format!("{:#?}", err).as_ref()),
					option,
				);

				retry(body, count - 1, RetryOption::Tor).await
			}
		};
	}

	let mut input: CheckEmailInput = body.clone().into();

	// If `with_proxy` and relevant ENV vars are set, we proxy.
	if let (RetryOption::Tor, Ok(proxy_host), Ok(proxy_port)) = (
		option,
		env::var("RCH_PROXY_HOST"),
		env::var("RCH_PROXY_PORT"),
	) {
		if let Ok(proxy_port) = proxy_port.parse::<u16>() {
			input.proxy(proxy_host, proxy_port);
		}
	}

	let result = ciee_check_email(&input)
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");

	// We return the last fetched result, if the retry count is exhausted.
	if count <= 1 {
		(ReacherOutput::Ciee(Box::new(result)), option)
	} else {
		match (&result.misc, &result.mx, &result.smtp) {
			(Err(error), _, _) => {
				// We log misc errors.
				sentry_util::error(
					format!("{:?}", error),
					Some(format!("{:#?}", result).as_ref()),
					option,
				);

				// We retry once again.
				retry(body, count - 1, option).await
			}
			(_, Err(error), _) => {
				// We log mx errors.
				sentry_util::error(
					format!("{:?}", error),
					Some(format!("{:#?}", result).as_ref()),
					option,
				);

				// We retry once again.
				retry(body, count - 1, option).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Permanent(response))))
				if has_smtp_permanent_errors(&response.message) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// We retry, once with Tor, once with Heroku...
				retry(body, count - 1, option.rotate()).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
				if has_smtp_transient_errors(&response.message) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// We retry, once with Tor, once with Heroku...
				retry(body, count - 1, option.rotate()).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Io(error))))
				if has_smtp_io_errors(error) =>
			{
				log::debug!(target: "reacher", "{}", error);
				// We retry, once with Tor, once with Heroku...
				retry(body, count - 1, option.rotate()).await
			}
			(_, _, Err(error)) => {
				// If it's a SMTP error we didn't catch above, we log to
				// Sentry, to be able to debug them better. We don't want to
				// spam Sentry and log all instances of the error, hence the
				// `count` check.
				sentry_util::error(
					format!("{:?}", error),
					Some(format!("{:#?}", result).as_ref()),
					option,
				);

				// We retry, once with Tor, once with heroku...
				retry(body, count - 1, option.rotate()).await
			}
			// If everything is ok, we just return the result.
			(Ok(_), Ok(_), Ok(_)) => (ReacherOutput::Ciee(Box::new(result)), option),
		}
	}
}

/// The main `check_email` function, on Serverless.
pub async fn check_email_serverless(body: ReacherInput) -> ReacherOutput {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time. The count is set to 5, so that we try thrice with
	// Tor, twice with Heroku, and thus bypass greylisting.
	let now = Instant::now();
	let (result, option) = retry(body, 5, RetryOption::Tor).await;

	// Note: This will not log if we made a request to Heroku.
	// FIXME: We should also log if we used RetryOption::Heroku.
	if let ReacherOutput::Ciee(value) = &result {
		sentry_util::info(
			format!("is_reachable={:?}", value.is_reachable),
			option,
			now.elapsed().as_millis(),
		);
	}

	result
}
