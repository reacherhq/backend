// Reacher - Email Verification
// Copyright (C) 2018-2020 Amaury Martiny

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your retry_option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! This module contains functions that check if the error's given by
//! `check-if-email-exists` are known errors, in which case we don't log them
//! to Sentry.

use super::RetryOption;
use crate::sentry_util;
use async_smtp::smtp::error::Error as AsyncSmtpError;
use check_if_email_exists::{smtp::SmtpError, CheckEmailOutput};
use std::io::Error as IoError;

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
    // Unable to add <EMAIL> because host 23.129.64.184 is listed on zen.spamhaus.org
    // 5.7.1 Service unavailable, Client host [23.129.64.184] blocked using Spamhaus.
    // 5.7.1 Email cannot be delivered. Reason: Email detected as Spam by spam filters.
    first_line.contains("spam") ||
    // host 23.129.64.216 is listed at combined.mail.abusix.zone (127.0.0.12,
    first_line.contains("abusix") ||
    // 5.7.1 <unknown[23.129.64.100]>: Client host rejected: Access denied
    first_line.contains("access denied") ||
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
    // 23.129.64.216 is not yet authorized to deliver mail from
    first_line.contains("not yet authorized")
}

/// Checks if the output from `check-if-email-exists` has a known error, in
/// which case we don't log to Sentry to avoid spamming it.
pub fn has_known_errors(result: &CheckEmailOutput, retry_option: RetryOption) -> bool {
	match (&result.misc, &result.mx, &result.smtp) {
		(Err(error), _, _) => {
			// We log misc errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				Some(retry_option),
			);

			true
		}
		(_, Err(error), _) => {
			// We log mx errors.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				Some(retry_option),
			);

			true
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Permanent(response))))
			if has_smtp_permanent_errors(&response.message) =>
		{
			log::debug!(target: "reacher", "{}", response.message[0]);

			true
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Transient(response))))
			if has_smtp_transient_errors(&response.message) =>
		{
			log::debug!(target: "reacher", "{}", response.message[0]);
			true
		}
		(_, _, Err(SmtpError::SmtpError(AsyncSmtpError::Io(error))))
			if has_smtp_io_errors(error) =>
		{
			log::debug!(target: "reacher", "{}", error);
			true
		}
		(_, _, Err(error)) => {
			// If it's a SMTP error we didn't catch above, we log to
			// Sentry, to be able to debug them better. We don't want to
			// spam Sentry and log all instances of the error, hence the
			// `count` check.
			sentry_util::error(
				format!("{:?}", error),
				Some(format!("{:#?}", result).as_ref()),
				Some(retry_option),
			);

			true
		}
		// If everything is ok, we just return the result.
		(Ok(_), Ok(_), Ok(_)) => false,
	}
}
