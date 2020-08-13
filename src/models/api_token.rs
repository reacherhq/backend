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

use super::schema::api_tokens;
use diesel::{pg::PgConnection, QueryResult, RunQueryDsl};
use uuid::Uuid;

#[derive(Associations, Debug, Identifiable, PartialEq, Queryable)]
#[belongs_to(super::user::User)]
#[table_name = "api_tokens"]
pub struct ApiToken {
	pub id: i32,
	pub api_token: Uuid,
	pub stripe_subscription_item: String,
	pub user_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "api_tokens"]
struct NewApiToken<'a> {
	pub stripe_subscription_item: &'a str,
	pub user_id: &'a Uuid,
}

/// Create an API token.
pub fn create_api_token<'a>(
	conn: &PgConnection,
	stripe_subscription_item: &'a str,
	user_id: &'a Uuid,
) -> QueryResult<ApiToken> {
	let new_api_token = NewApiToken {
		stripe_subscription_item,
		user_id,
	};

	diesel::insert_into(api_tokens::table)
		.values(&new_api_token)
		.get_result::<ApiToken>(conn)
}

/// Get an API token by its UUID.
pub fn find_one_by_api_token<'a>(conn: &PgConnection, token: &Uuid) -> QueryResult<ApiToken> {
	use super::schema::api_tokens::dsl::*;
	use diesel::prelude::*;

	api_tokens.filter(api_token.eq(token)).first(conn)
}
