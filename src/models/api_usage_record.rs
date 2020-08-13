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

use super::schema::api_usage_records;
use chrono::NaiveDateTime;
use diesel::{pg::PgConnection, QueryResult, RunQueryDsl};

#[derive(Associations, Debug, Identifiable, PartialEq, Queryable)]
#[belongs_to(super::api_token::ApiToken)]
#[table_name = "api_usage_records"]
pub struct ApiUsageRecord {
	pub id: i32,
	pub api_token_id: i32,
	pub method: String,
	pub endpoint: String,
	pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "api_usage_records"]
struct NewApiUsageRecord<'a> {
	pub api_token_id: i32,
	pub method: &'a str,
	pub endpoint: &'a str,
}

/// Create an API usage record.
pub fn create_api_usage_record<'a>(
	conn: &PgConnection,
	api_token_id: i32,
	method: &'a str,
	endpoint: &'a str,
) -> QueryResult<ApiUsageRecord> {
	let new_record = NewApiUsageRecord {
		api_token_id,
		method,
		endpoint,
	};

	diesel::insert_into(api_usage_records::table)
		.values(&new_record)
		.get_result::<ApiUsageRecord>(conn)
}

/// Create an API usage record.
pub fn get_api_usage_records_by_api_token<'a>(
	conn: &PgConnection,
	find_api_token_id: i32,
) -> QueryResult<Vec<ApiUsageRecord>> {
	use super::schema::api_usage_records::dsl::*;
	use diesel::prelude::*;

	api_usage_records
		.filter(api_token_id.eq(find_api_token_id))
		.load(conn)
}
