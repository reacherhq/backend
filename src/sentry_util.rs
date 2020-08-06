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

//! Helper functions to send events to Sentry.

use crate::routes::check_email::post::RetryOption;
use sentry::protocol::{Event, Level};
use std::{collections::BTreeMap, env};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Helper function to send an Info event to Sentry.
pub fn info(message: String, option: RetryOption, duration: u128) {
	log::info!("Sending to Sentry: {}", message);

	let mut extra = BTreeMap::new();

	extra.insert("duration".into(), duration.to_string().into());
	extra.insert("proxy_option".into(), option.to_string().into());

	sentry::capture_event(Event {
		extra,
		level: Level::Info,
		message: Some(message),
		// FIXME It seams that this doesn't work on Sentry, so I added it in
		// the `extra` field above too.
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}

/// Helper function to send an Error event to Sentry.
pub fn error(message: String, result: Option<&str>, retry_option: Option<RetryOption>) {
	log::debug!("Sending to Sentry: {}", message);
	let mut extra = BTreeMap::new();

	if let Some(result) = result {
		extra.insert("CheckEmailOutput".into(), result.into());
	}
	if let Some(retry_option) = retry_option {
		extra.insert("proxy_option".into(), retry_option.to_string().into());
	}

	sentry::capture_event(Event {
		extra,
		level: Level::Error,
		message: Some(message),
		// FIXME It seams that this doesn't work on Sentry, so I added it in
		// the `extra` field above too.
		release: Some(CARGO_PKG_VERSION.into()),
		..Default::default()
	});
}
