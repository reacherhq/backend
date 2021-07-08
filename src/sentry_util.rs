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

//! Helper functions to send events to Sentry.

use crate::routes::check_email::post::RetryOption;
use lazy_static::lazy_static;
use regex::Regex;
use sentry::protocol::{Event, Level, Value};
use std::{collections::BTreeMap, env};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
	// Regex to extract emails from a string.
	static ref RE: Regex =
		Regex::new(r"[a-zA-Z0-9._-]+@(?P<domain>[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)")
			.unwrap();
}

/// Setup Sentry.
pub fn setup_sentry() -> sentry::ClientInitGuard {
	// Use an empty string if we don't have any env variable for sentry. Sentry
	// will just silently ignore.
	let sentry = sentry::init(env::var("RCH_SENTRY_DSN").unwrap_or_else(|_| "".into()));
	if sentry.is_enabled() {
		log::info!(target: "reacher", "Sentry is successfully set up.")
	}

	sentry
}

/// If HEROKU_APP_NAME environment variable is set, add it to the sentry `extra`
/// properties.
fn add_heroku_app_name(mut extra: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
	if let Ok(heroku_app_name) = env::var("HEROKU_APP_NAME") {
		extra.insert("HEROKU_APP_NAME".into(), heroku_app_name.into());
	}

	extra
}

/// Helper function to send an Info event to Sentry. We use these events for
/// analytics purposes (I know, Sentry shouldn't be used for that...).
/// TODO https://github.com/reacherhq/backend/issues/207
pub fn metrics(message: String, retry_option: RetryOption, duration: u128, domain: &str) {
	log::info!("Sending info to Sentry: {}", message);

	let mut extra = BTreeMap::new();

	extra.insert("duration".into(), duration.to_string().into());
	extra.insert("retry_option".into(), retry_option.to_string().into());
	extra.insert("domain".into(), domain.into());
	extra = add_heroku_app_name(extra);

	sentry::capture_event(Event {
		extra,
		level: Level::Info,
		message: Some(message),
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}

/// Helper function to send an Error event to Sentry.
pub fn error(message: String, result: Option<&str>, retry_option: Option<RetryOption>) {
	let redacted_message = redact(message.as_str());
	log::debug!("Sending error to Sentry: {}", redacted_message);

	let mut extra = BTreeMap::new();
	if let Some(result) = result {
		extra.insert("CheckEmailOutput".into(), redact(result).into());
	}
	if let Some(retry_option) = retry_option {
		extra.insert("retry_option".into(), retry_option.to_string().into());
	}
	extra = add_heroku_app_name(extra);

	sentry::capture_event(Event {
		extra,
		level: Level::Error,
		message: Some(redacted_message),
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}

/// Function to parse emails inside a text, and replace them with
/// `*@domain.com` for privacy reasons.
fn redact(input: &str) -> String {
	let result = RE.replace_all(input, "*@$domain");
	result.into()
}

#[cfg(test)]
mod tests {
	use super::redact;

	#[test]
	fn test_redact() {
		assert_eq!("*@gmail.com", redact("someone@gmail.com"));
		assert_eq!(
			"my email is *@gmail.com.",
			redact("my email is someone@gmail.com.")
		);
		assert_eq!(
			"my email is *@gmail.com., I repeat, my email is *@gmail.com.",
			redact("my email is someone@gmail.com., I repeat, my email is someone@gmail.com.")
		);
		assert_eq!("someone @ gmail . com", redact("someone @ gmail . com"));
	}
}
