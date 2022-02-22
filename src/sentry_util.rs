// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

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
//!
//! This module also contains functions that check if the error's given by
//! `check-if-email-exists` are known errors, in which case we don't log them
//! to Sentry.

use super::sentry_util;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{smtp::SmtpError, CheckEmailOutput};
use sentry::protocol::{Event, Level, Value};
use std::io::Error as IoError;
use std::{collections::BTreeMap, env};

pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub fn metrics(message: String, duration: u128, domain: &str) {
	log::info!("Sending info to Sentry: {}", message);

	let mut extra = BTreeMap::new();

	extra.insert("duration".into(), duration.to_string().into());
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

/// Helper function to send an Error event to Sentry. We redact all sensitive
/// info before sending to Sentry, but removing all instances of `username`.
pub fn error(message: String, result: Option<&str>, username: &str) {
	let redacted_message = redact(message.as_str(), username);
	log::debug!("Sending error to Sentry: {}", redacted_message);

	let mut extra = BTreeMap::new();
	if let Some(result) = result {
		extra.insert("CheckEmailOutput".into(), redact(result, username).into());
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

/// Function to replace all usernames from email, and replace them with
/// `***@domain.com` for privacy reasons.
fn redact(input: &str, username: &str) -> String {
	input.replace(username, "***")
}

/// Check if the message contains known SMTP IO errors.
fn has_smtp_io_errors(error: &IoError) -> bool {
	// code: 104, kind: ConnectionReset, message: "Connection reset by peer",
	error.raw_os_error() == Some(104) ||
	// kind: Other, error: "incomplete",
	error.to_string() == "incomplete"
}

/// Check if the message contains known SMTP Permanent errors.
fn has_smtp_permanent_errors(message: &[String]) -> bool {
	let first_line = message[0].to_lowercase();

	// 5.7.1 IP address blacklisted by recipient
	// 5.7.1 Service unavailable; Client host [147.75.45.223] is blacklisted. Visit https://www.sophos.com/en-us/threat-center/ip-lookup.aspx?ip=147.75.45.223 to request delisting
	// 5.3.0 <aaro.peramaa@helsinki.fi>... Mail from 147.75.45.223 rejected by Abusix blacklist
	first_line.contains("blacklist") ||
	// Rejected because 23.129.64.213 is in a black list at b.barracudacentral.org
	first_line.contains("black list") ||
	// 5.7.1 Recipient not authorized, your IP has been found on a block list
	first_line.contains("block list") ||
	// Unable to add <EMAIL> because host 23.129.64.184 is listed on zen.spamhaus.org
	// 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
	// 5.7.1 Email cannot be delivered. Reason: Email detected as Spam by spam filters.
	first_line.contains("spam") ||
	// host 23.129.64.216 is listed at combined.mail.abusix.zone (127.0.0.12,
	first_line.contains("abusix") ||
	// 5.7.1 Relaying denied. IP name possibly forged [45.154.35.252]
	// 5.7.1 Relaying denied: You must check for new mail before sending mail. [23.129.64.216]
	first_line.contains("relaying denied") ||
	// 5.7.1 <unknown[23.129.64.100]>: Client host rejected: Access denied
	first_line.contains("access denied") ||
	// sorry, mail from your location [5.79.109.48] is administratively denied (#5.7.1)
	first_line.contains("administratively denied") ||
	// 5.7.606 Access denied, banned sending IP [23.129.64.216]
	first_line.contains("banned") ||
	// Blocked - see https://ipcheck.proofpoint.com/?ip=23.129.64.192
	// 5.7.1 Mail from 23.129.64.183 has been blocked by Trend Micro Email Reputation Service.
	first_line.contains("blocked") ||
	// Connection rejected by policy [7.3] 38206, please visit https://support.symantec.com/en_US/article.TECH246726.html for more details about this error message.
	first_line.contains("connection rejected") ||
	// 5.7.1 Client host rejected: cannot find your reverse hostname, [23.129.64.184]
	first_line.contains("cannot find your reverse hostname") ||
	// csi.mimecast.org Poor Reputation Sender. - https://community.mimecast.com/docs/DOC-1369#550 [6ATVl4DjOvSA6XNsWGoUFw.us31]
	first_line.contains("poor reputation") ||
	// JunkMail rejected - (gmail.com) [193.218.118.140]:46615 is in an RBL: http://www.barracudanetworks.com/reputation/?pr=1&ip=193.218.118.140
	first_line.contains("junkmail")||
	// Your access to this mail system has been rejected due to the sending MTA\'s poor reputation. If you believe that this failure is in error, please contact the intended recipient via alternate means.
	(message.len() >= 2 && message[1].contains("rejected"))
}

/// Check if the message contains known SMTP Transient errors.
fn has_smtp_transient_errors(message: &[String]) -> bool {
	let first_line = message[0].to_lowercase();

	// Blocked - see https://www.spamcop.net/bl.shtml?23.129.64.211
	first_line.contains("blocked") ||
	// 4.7.1 <EMAIL>: Relay access denied
	first_line.contains("access denied") ||
	// 4.7.25 Client host rejected: cannot find your hostname, [147.75.45.223]
	// 4.7.1 Client host rejected: cannot find your reverse hostname, [147.75.45.223]
	first_line.contains("host rejected") ||
	// relay not permitted!
	first_line.contains("relay not permitted") ||
	// You dont seem to have a reverse dns entry. Come back later. You are greylisted for 20 minutes. See http://www.fsf.org/about/systems/greylisting
	first_line.contains("reverse dns entry") ||
	// 23.129.64.216 is not yet authorized to deliver mail from
	first_line.contains("not yet authorized") ||
	// 4.3.2 Please try again later
	first_line.contains("try again") ||
	// Temporary local problem - please try later
	first_line.contains("try later")
}

/// Checks if the output from `check-if-email-exists` has a known error, in
/// which case we don't log to Sentry to avoid spamming it.
pub fn log_unknown_errors(result: &CheckEmailOutput) {
	match (&result.misc, &result.mx, &result.smtp) {
		(Err(error), _, _) => {
			// We log misc errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		(_, Err(error), _) => {
			// We log mx errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		(_, _, Err(SmtpError::HeloError(AsyncSmtpError::Permanent(response))))
		| (_, _, Err(SmtpError::ConnectError(AsyncSmtpError::Permanent(response))))
		| (_, _, Err(SmtpError::ConnectWithStreamError(AsyncSmtpError::Permanent(response))))
		| (_, _, Err(SmtpError::MailFromError(AsyncSmtpError::Permanent(response))))
		| (_, _, Err(SmtpError::RcptToError(AsyncSmtpError::Permanent(response))))
		| (_, _, Err(SmtpError::CloseError(AsyncSmtpError::Permanent(response))))
			if has_smtp_permanent_errors(&response.message) =>
		{
			log::debug!(target: "reacher", "Permanent error: {}", response.message[0]);
		}
		(_, _, Err(SmtpError::HeloError(AsyncSmtpError::Transient(response))))
		| (_, _, Err(SmtpError::ConnectError(AsyncSmtpError::Transient(response))))
		| (_, _, Err(SmtpError::ConnectWithStreamError(AsyncSmtpError::Transient(response))))
		| (_, _, Err(SmtpError::MailFromError(AsyncSmtpError::Transient(response))))
		| (_, _, Err(SmtpError::RcptToError(AsyncSmtpError::Transient(response))))
		| (_, _, Err(SmtpError::CloseError(AsyncSmtpError::Transient(response))))
			if has_smtp_transient_errors(&response.message) =>
		{
			log::debug!(target: "reacher", "Transient error: {}", response.message[0]);
		}
		(_, _, Err(SmtpError::HeloError(AsyncSmtpError::Io(err))))
		| (_, _, Err(SmtpError::ConnectError(AsyncSmtpError::Io(err))))
		| (_, _, Err(SmtpError::ConnectWithStreamError(AsyncSmtpError::Io(err))))
		| (_, _, Err(SmtpError::MailFromError(AsyncSmtpError::Io(err))))
		| (_, _, Err(SmtpError::RcptToError(AsyncSmtpError::Io(err))))
		| (_, _, Err(SmtpError::CloseError(AsyncSmtpError::Io(err))))
			if has_smtp_io_errors(err) =>
		{
			log::debug!(target: "reacher", "Io error: {}", err);
		}
		(_, _, Err(error)) => {
			// If it's a SMTP error we didn't catch above, we log to
			// Sentry, to be able to debug them better. We don't want to
			// spam Sentry and log all instances of the error, hence the
			// `count` check.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				result.syntax.username.as_str(),
			);
		}
		// If everything is ok, we just return the result.
		(Ok(_), Ok(_), Ok(_)) => {}
	}
}

#[cfg(test)]
mod tests {
	use super::redact;

	#[test]
	fn test_redact() {
		assert_eq!("***@gmail.com", redact("someone@gmail.com", "someone"));
		assert_eq!(
			"my email is ***@gmail.com.",
			redact("my email is someone@gmail.com.", "someone")
		);
		assert_eq!(
			"my email is ***@gmail.com., I repeat, my email is ***@gmail.com.",
			redact(
				"my email is someone@gmail.com., I repeat, my email is someone@gmail.com.",
				"someone"
			)
		);
		assert_eq!(
			"*** @ gmail . com",
			redact("someone @ gmail . com", "someone")
		);
		assert_eq!("*** is here.", redact("someone is here.", "someone"));
	}
}
