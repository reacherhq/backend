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

#[derive(Associations, Debug, Identifiable, PartialEq, Queryable)]
#[belongs_to(super::user::User)]
#[table_name = "api_tokens"]
pub struct ApiToken {
	pub id: String,
	pub api_token: String,
	pub stripe_subscription_item: String,
	pub user_id: String,
}
