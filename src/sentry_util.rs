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

use super::handlers::RetryOption;
use check_if_email_exists::CheckEmailOutput;
use sentry::protocol::{Event, Level};
use std::{collections::BTreeMap, env};

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper function to send an Info event to Sentry.
pub fn info(message: String, option: RetryOption, duration: u128) {
	let mut extra = BTreeMap::new();
	if let Ok(fly_alloc_id) = env::var("FLY_ALLOC_ID") {
		extra.insert("FLY_ALLOC_ID".into(), fly_alloc_id.into());
	}
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
pub fn error(message: String, result: &CheckEmailOutput, option: RetryOption) {
	log::debug!("{}", message);

	let mut extra = BTreeMap::new();
	extra.insert("CheckEmailOutput".into(), format!("{:#?}", result).into());
	if let Ok(fly_alloc_id) = env::var("FLY_ALLOC_ID") {
		extra.insert("FLY_ALLOC_ID".into(), fly_alloc_id.into());
	}
	extra.insert("proxy_option".into(), option.to_string().into());

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
