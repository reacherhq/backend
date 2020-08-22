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
