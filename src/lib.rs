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

pub mod heroku;
mod saasify_secret;
mod sentry_util;
pub mod serverless;

use check_if_email_exists::{CheckEmailInput, CheckEmailOutput};
use sentry_util::CARGO_PKG_VERSION;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::{env, fmt};

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

/// Setup logging and Sentry.
pub fn setup_sentry() -> sentry::ClientInitGuard {
	log::info!(target: "reacher", "Running Reacher v{}", CARGO_PKG_VERSION);

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
