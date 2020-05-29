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

use super::sentry_util;
use async_recursion::async_recursion;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{
	check_email as ciee_check_email, smtp::SmtpError, CheckEmailInput, CheckEmailOutput,
};
use http_types::headers::HeaderName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, env, fmt, time::Instant};
use warp::http::StatusCode;

/// JSON body for POST /check_email
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReacherInput {
	from_email: Option<String>,
	hello_name: Option<String>,
	to_email: String,
}

impl Into<CheckEmailInput> for ReacherInput {
	fn into(self) -> CheckEmailInput {
		// Create ReacherInput for check_if_email_exists from body
		let mut input = CheckEmailInput::new(vec![self.to_email]);
		input
			.from_email(self.from_email.unwrap_or_else(|| {
				env::var("RCH_FROM_EMAIL").unwrap_or_else(|_| "user@example.org".into())
			}))
			.hello_name(self.hello_name.unwrap_or_else(|| "gmail.com".into()));

		input
	}
}

/// Response for POST /check_email. This is mainly an internal type, and both
/// serialize to the same value.
#[derive(Serialize)]
enum ReacherOutput {
	Ciee(CheckEmailOutput),
	Json(Value),
}

/// This option represents how we should execute the SMTP connection to check
/// an email.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryOption {
	/// Use Tor to connect to the SMTP server.
	Tor,
	/// Direct connection to the SMTP server.
	Direct,
	/// Send a HTTP request to Heroku, which will connect to the SMTP server.
	Heroku,
}

impl RetryOption {
	fn rotate(&self) -> Self {
		match self {
			RetryOption::Tor => RetryOption::Direct,
			RetryOption::Direct => RetryOption::Heroku,
			RetryOption::Heroku => RetryOption::Direct,
		}
	}
}

impl fmt::Display for RetryOption {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

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
async fn check_fly(
	body: ReacherInput,
	count: u8,
	option: RetryOption,
) -> (ReacherOutput, RetryOption) {
	log::debug!(target: "reacher", "Retry #{} for {}, with proxy {:?}", count, body.to_email, option);

	// If we're using Heroku option, then we make a HTTP call to Heroku.
	if option == RetryOption::Heroku {
		let result: Value = match surf::post("https://reacher-us-1.herokuapp.com/check_email")
			.set_header("Content-Type".parse().unwrap(), "application/json")
			.set_header(
				"x-saasify-proxy-secret",
				env::var("RCH_SAASIFY_SECRET").unwrap_or_else(|_| "reacher_dev_secret".into()),
			)
			.body_json(&body)
			.expect("We made sure the body is correct. qed.")
			.recv_json()
			.await
		{
			Ok(result) => result,
			Err(_) => unreachable!(),
		};

		return (ReacherOutput::Json(result), option);
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
		(ReacherOutput::Ciee(result), option)
	} else {
		match (&result.misc, &result.mx, &result.smtp) {
			(Err(error), _, _) => {
				// We log misc errors.
				sentry_util::error(format!("{:?}", error), &result, option);

				// We retry once again.
				check_fly(body, count - 1, option).await
			}
			(_, Err(error), _) => {
				// We log mx errors.
				sentry_util::error(format!("{:?}", error), &result, option);

				// We retry once again.
				check_fly(body, count - 1, option).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Permanent(response))))
				if (
					// Unable to add <email> because host 23.129.64.184 is listed on zen.spamhaus.org
					// 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
					// 5.7.1 Email cannot be delivered. Reason: Email detected as Spam by spam filters.
					response.message[0].to_lowercase().contains("spam") ||
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
				// We retry, once with Tor, once with heroku, once direct...
				check_fly(body, count - 1, option.rotate()).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
				if (
					// 4.7.1 <email>: Relay access denied
					response.message[0].to_lowercase().contains("access denied") ||
					// relay not permitted!
					response.message[0].to_lowercase().contains("relay not permitted") ||
					// 23.129.64.216 is not yet authorized to deliver mail from
					response.message[0].to_lowercase().contains("not yet authorized")
				) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// We retry, once with Tor, once with heroku, once direct...
				check_fly(body, count - 1, option.rotate()).await
			}
			(_, _, Err(error)) => {
				// If it's a SMTP error we didn't catch above, we log to
				// Sentry, to be able to debug them better. We don't want to
				// spam Sentry and log all instances of the error, hence the
				// `count` check.
				sentry_util::error(format!("{:?}", error), &result, option);

				// We retry, once with Tor, once with heroku, once direct...
				check_fly(body, count - 1, option.rotate()).await
			}
			// If everything is ok, we just return the result.
			(Ok(_), Ok(_), Ok(_)) => (ReacherOutput::Ciee(result), option),
		}
	}
}

/// If we're on Heroku, then we just do a simple check.
async fn check_heroku(body: ReacherInput) -> (ReacherOutput, RetryOption) {
	(
		ReacherOutput::Ciee(
			ciee_check_email(&body.into())
				.await
				.pop()
				.expect("The input has one element, so does the output. qed."),
		),
		RetryOption::Heroku,
	)
}

/// We deploy this same code on Fly and on Heroku. Depending on which provider
/// we're calling this code from, we execute a different logic.
async fn check(body: ReacherInput) -> (ReacherOutput, RetryOption) {
	// Detect if we're on heroku.
	let is_fly = env::var("FLY_ALLOC_ID").is_ok();
	if is_fly {
		check_fly(body, 3, RetryOption::Tor).await
	} else {
		check_heroku(body).await
	}
}

/// Given an email address (and optionally some additional configuration
/// options), return if email verification details as given by
/// `check_if_email_exists`.
pub async fn check_email(_: (), body: ReacherInput) -> Result<impl warp::Reply, Infallible> {
	// Run `ciee_check_email` function 4 times max. Also measure the
	// verification time.
	let now = Instant::now();
	let (result, option) = check(body).await;
	// FIXME Also log results from Heroku.
	if let ReacherOutput::Ciee(value) = &result {
		sentry_util::info(
			format!("is_reachable={:?}", value.is_reachable),
			option,
			now.elapsed().as_millis(),
		);
	}

	Ok(warp::reply::with_status(
		warp::reply::json(&result),
		StatusCode::OK,
	))
}
