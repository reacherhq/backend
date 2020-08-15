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

use super::schema::users;
use diesel::{pg::PgConnection, QueryResult, RunQueryDsl};
use uuid::Uuid;

#[derive(Debug, Identifiable, PartialEq, Queryable)]
#[table_name = "users"]
pub struct User {
	pub id: Uuid,
	pub stripe_customer: String,
}

#[derive(Insertable)]
#[table_name = "users"]
struct NewUser<'a> {
	pub stripe_customer: &'a str,
}

/// Create a User.
pub fn create_user<'a>(conn: &PgConnection, stripe_customer: &'a str) -> QueryResult<User> {
	let new_user = NewUser { stripe_customer };

	diesel::insert_into(users::table)
		.values(&new_user)
		.get_result::<User>(conn)
}

/// Get a User by strip_customer.
pub fn get_user_by_stripe_customer<'a>(
	conn: &PgConnection,
	customer: &'a str,
) -> QueryResult<User> {
	use super::schema::users::dsl::*;
	use diesel::prelude::*;

	users.filter(stripe_customer.eq(customer)).first(conn)
}

/// Delete a User.
pub fn delete_user(conn: &PgConnection, user_id: &Uuid) -> QueryResult<usize> {
	use super::schema::users::dsl::*;
	use diesel::prelude::*;

	diesel::delete(users.filter(id.eq(user_id))).execute(conn)
}
