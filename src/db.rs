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

use diesel::{
	r2d2::{ConnectionManager, Pool},
	PgConnection,
};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Connect to the db, and create a connection pool.
pub fn connect_db(database_url: &str) -> PgPool {
	let manager = ConnectionManager::<PgConnection>::new(database_url);

	Pool::builder()
		.build(manager)
		.unwrap_or_else(|_| panic!("Failed to create pool with DB {}", database_url))
}
