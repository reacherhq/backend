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

use crate::errors::ReacherResponseError;
use futures::{try_join, Future, FutureExt};
use std::fmt;
use warp::{http, reject, Rejection};

/// Convert a psql error to a warp rejection.
pub fn pg_to_warp_error<T>(err: T) -> Rejection
where
	T: fmt::Display,
{
	reject::custom(ReacherResponseError::new(
		http::StatusCode::INTERNAL_SERVER_ERROR,
		err.to_string(),
	))
}

/// Converts a Result<T, E> to a Result<E, T>.
fn reverse_result<T, E>(res: Result<T, E>) -> Result<E, T> {
	match res {
		Ok(v) => Err(v),
		Err(err) => Ok(err),
	}
}

/// Given 2 futures that return a Result, return either the value of first
/// future to complete, or a tuple containing the 2 errors.
pub async fn race_future2<F, T, E>(fut1: F, fut2: F) -> Result<T, (E, E)>
where
	F: Future<Output = Result<T, E>>,
{
	let rev_fut1 = fut1.map(reverse_result);
	let rev_fut2 = fut2.map(reverse_result);

	match try_join!(rev_fut1, rev_fut2) {
		Ok((err1, err2)) => Err((err1, err2)),
		Err(res) => Ok(res),
	}
}
