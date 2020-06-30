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

pub mod saasify_secret;
pub mod sentry_util;

use async_recursion::async_recursion;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{
	check_email as ciee_check_email, smtp::SmtpError, CheckEmailInput, CheckEmailOutput, Reachable,
};
use saasify_secret::get_saasify_secret;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::{env, fmt, time::Instant};

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
#[serde(untagged)]
pub enum ReacherOutput {
	Ciee(Box<CheckEmailOutput>), // Large variant, boxing the large fields to reduce the total size of the enum.
	Json(Value),
}

/// This option represents how we should execute the SMTP connection to check
/// an email.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryOption {
	/// Use Tor to connect to the SMTP server.
	Tor,
	/// Send a HTTP request to Heroku, which will connect to the SMTP server
	/// directly.
	Heroku,
}

impl RetryOption {
	/// In our retry mechanism, we rotate the way we connect to the SMTP
	/// server.
	fn rotate(self) -> Self {
		match self {
			RetryOption::Tor => RetryOption::Heroku,
			RetryOption::Heroku => RetryOption::Tor,
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
async fn check_serverless(
	body: ReacherInput,
	count: u8,
	option: RetryOption,
) -> (ReacherOutput, RetryOption) {
	log::info!(target: "reacher", "Retry #{}, with proxy {:?}", count, option);

	// If we're using Heroku option, then we make a HTTP call to Heroku.
	if option == RetryOption::Heroku {
		return match surf::post("https://reacher-us-1.herokuapp.com/check_email")
			.set_header("Content-Type", "application/json")
			.set_header("x-saasify-proxy-secret", get_saasify_secret())
			.body_json(&body)
			.expect("We made sure the body is correct. qed.")
			.recv_json()
			.await
		{
			Ok(result) => (ReacherOutput::Json(result), option),
			Err(err) => {
				sentry_util::error(
					format!("Heroku response error: {}", err.to_string()),
					None,
					option,
				);

				check_serverless(body, count - 1, RetryOption::Tor).await
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
				sentry_util::error(format!("{:?}", error), Some(&result), option);

				// We retry once again.
				check_serverless(body, count - 1, option).await
			}
			(_, Err(error), _) => {
				// We log mx errors.
				sentry_util::error(format!("{:?}", error), Some(&result), option);

				// We retry once again.
				check_serverless(body, count - 1, option).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Permanent(response))))
				if (
					// 5.7.1 IP address blacklisted by recipient
					// 5.7.1 Service unavailable; Client host [147.75.45.223] is blacklisted. Visit https://www.sophos.com/en-us/threat-center/ip-lookup.aspx?ip=147.75.45.223 to request delisting
					// 5.3.0 <aaro.peramaa@helsinki.fi>... Mail from 147.75.45.223 rejected by Abusix blacklist
					response.message[0].to_lowercase().contains("blacklist") ||
					// Unable to add <EMAIL> because host 23.129.64.184 is listed on zen.spamhaus.org
					// 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
					// 5.7.1 Email cannot be delivered. Reason: Email detected as Spam by spam filters.
					response.message[0].to_lowercase().contains("spam") ||
					// 5.7.1 <unknown[23.129.64.100]>: Client host rejected: Access denied
					response.message[0].to_lowercase().contains("access denied") ||
					// 5.7.606 Access denied, banned sending IP [23.129.64.216]
					response.message[0].to_lowercase().contains("banned") ||
					// Blocked - see https://ipcheck.proofpoint.com/?ip=23.129.64.192
					// 5.7.1 Mail from 23.129.64.183 has been blocked by Trend Micro Email Reputation Service.
					response.message[0].to_lowercase().contains("blocked") ||
					// Connection rejected by policy [7.3] 38206, please visit https://support.symantec.com/en_US/article.TECH246726.html for more details about this error message.
					response.message[0].to_lowercase().contains("connection rejected") ||
					// 5.7.1 Client host rejected: cannot find your reverse hostname, [23.129.64.184]
					response.message[0].to_lowercase().contains("cannot find your reverse hostname") ||
					// Your access to this mail system has been rejected due to the sending MTA\'s poor reputation. If you believe that this failure is in error, please contact the intended recipient via alternate means.
					(response.message.len() >= 2 && response.message[1].to_lowercase().contains("rejected"))
				) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// We retry, once with Tor, once with Heroku...
				check_serverless(body, count - 1, option.rotate()).await
			}
			(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
				if (
					// Blocked - see https://www.spamcop.net/bl.shtml?23.129.64.211
					response.message[0].to_lowercase().contains("blocked") ||
					// 4.7.1 <EMAIL>: Relay access denied
					response.message[0].to_lowercase().contains("access denied") ||
					// 4.7.25 Client host rejected: cannot find your hostname, [147.75.45.223]
					// 4.7.1 Client host rejected: cannot find your reverse hostname, [147.75.45.223]
					response.message[0].to_lowercase().contains("host rejected") ||
					// relay not permitted!
					response.message[0].to_lowercase().contains("relay not permitted") ||
					// 23.129.64.216 is not yet authorized to deliver mail from
					response.message[0].to_lowercase().contains("not yet authorized")
				) =>
			{
				log::debug!(target: "reacher", "{}", response.message[0]);
				// We retry, once with Tor, once with Heroku...
				check_serverless(body, count - 1, option.rotate()).await
			}
			(_, _, Err(error)) => {
				// If it's a SMTP error we didn't catch above, we log to
				// Sentry, to be able to debug them better. We don't want to
				// spam Sentry and log all instances of the error, hence the
				// `count` check.
				sentry_util::error(format!("{:?}", error), Some(&result), option);

				// We retry, once with Tor, once with heroku...
				check_serverless(body, count - 1, option.rotate()).await
			}
			// If everything is ok, we just return the result.
			(Ok(_), Ok(_), Ok(_)) => (ReacherOutput::Ciee(Box::new(result)), option),
		}
	}
}

/// If we're on Heroku, then we just do a simple check.
async fn check_heroku(body: ReacherInput) -> (ReacherOutput, RetryOption) {
	let result = ciee_check_email(&body.into())
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");

	// If we got Unknown, log it.
	// FIXME Better error message? For now, heroku errors should be quite rare,
	// so it's still okay.
	if result.is_reachable == Reachable::Unknown {
		sentry_util::error("heroku error".into(), Some(&result), RetryOption::Heroku);
	}

	(ReacherOutput::Ciee(Box::new(result)), RetryOption::Heroku)
}

/// The main `check_email` function, on Heroku.
pub async fn check_email_heroku(body: ReacherInput) -> ReacherOutput {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();
	let (result, option) = check_heroku(body).await;

	// This will only log the Heroku verification.
	if let ReacherOutput::Ciee(value) = &result {
		sentry_util::info(
			format!("is_reachable={:?}", value.is_reachable),
			option,
			now.elapsed().as_millis(),
		);
	}

	result
}

/// The main `check_email` function, on Serverless.
pub async fn check_email_serverless(body: ReacherInput) -> ReacherOutput {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time. The count is set to 4, so that we try twice with Tor,
	// twice with Heroku, and thus bypass greylisting.
	let now = Instant::now();
	let (result, option) = check_serverless(body, 4, RetryOption::Tor).await;

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

/// Setup logging and Sentry.
pub fn setup_sentry() -> sentry::ClientInitGuard {
	// Use an empty string if we don't have any env variable for sentry. Sentry
	// will just silently ignore.
	let sentry = sentry::init(env::var("RCH_SENTRY_DSN").unwrap_or_else(|_| "".into()));
	if sentry.is_enabled() {
		log::info!(target: "reacher", "Sentry is successfully set up.")
	}

	sentry
}

/// Struct describing an error response.
#[derive(Debug)]
pub struct ReacherOutputError<T> {
	error: T,
}

impl<T> ReacherOutputError<T> {
	pub fn new(error: T) -> Self {
		ReacherOutputError { error }
	}
}

impl<T> Serialize for ReacherOutputError<T>
where
	T: fmt::Display,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(Some(1))?;
		map.serialize_entry("error", &format!("{}", self.error))?;
		map.end()
	}
}
