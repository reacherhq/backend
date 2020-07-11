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

use crate::{sentry_util, ReacherInput, ReacherOutput, RetryOption};
use check_if_email_exists::check_email as ciee_check_email;

use std::time::Instant;

/// The main `check_email` function, on Heroku.
pub async fn check_email_heroku(body: ReacherInput) -> ReacherOutput {
	// Run `ciee_check_email` with retries if necessary. Also measure the
	// verification time.
	let now = Instant::now();
	let result = ciee_check_email(&body.into())
		.await
		.pop()
		.expect("The input has one element, so does the output. qed.");
	let result = ReacherOutput::Ciee(Box::new(result));

	// This will only log the Heroku verification.
	if let ReacherOutput::Ciee(value) = &result {
		sentry_util::info(
			format!("is_reachable={:?}", value.is_reachable),
			RetryOption::Heroku,
			now.elapsed().as_millis(),
		);
	}

	result
}
