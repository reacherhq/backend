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

//! This module contains functions that check if the error's given by
//! `check-if-email-exists` are known errors, in which case we don't log them
//! to Sentry.

use std::io::Error as IoError;

/// Check if the message contains known SMTP IO errors.
pub fn has_smtp_io_errors(error: &IoError) -> bool {
	// code: 104, kind: ConnectionReset, message: "Connection reset by peer",
	error.raw_os_error() == Some(104) ||
    // kind: Other, error: "incomplete",
    error.to_string() == "incomplete"
}

/// Check if the message contains known SMTP Permanent errors.
pub fn has_smtp_permanent_errors(message: &Vec<String>) -> bool {
	let first_line = message[0].to_lowercase();

	// 5.7.1 IP address blacklisted by recipient
	// 5.7.1 Service unavailable; Client host [147.75.45.223] is blacklisted. Visit https://www.sophos.com/en-us/threat-center/ip-lookup.aspx?ip=147.75.45.223 to request delisting
	// 5.3.0 <aaro.peramaa@helsinki.fi>... Mail from 147.75.45.223 rejected by Abusix blacklist
	first_line.contains("blacklist") ||
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
    // Your access to this mail system has been rejected due to the sending MTA\'s poor reputation. If you believe that this failure is in error, please contact the intended recipient via alternate means.
    (message.len() >= 2 && message[1].contains("rejected"))
}

/// Check if the message contains known SMTP Transient errors.
pub fn has_smtp_transient_errors(message: &Vec<String>) -> bool {
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
