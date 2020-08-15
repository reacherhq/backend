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

use rand::{distributions::Alphanumeric, Rng};
use reacher_backend::{
	db::PgPool,
	models::{
		api_token::{create_api_token, ApiToken},
		user::{create_user, delete_user, User},
	},
};
use uuid::Uuid;

/// Clean up the database after a test.
pub fn teardown_pool(pool: PgPool, user_ids: Vec<&Uuid>) {
	let connection = pool
		.get()
		.expect("DB pool is expected to be defined in tests. qed.");

	// Cascading should take care of deleting everything in the DB.
	for user_id in user_ids.into_iter() {
		delete_user(&connection, user_id)
			.expect(format!("User {} exists, can be deleted. qed.", user_id).as_str());
	}
}

/// Create a user, with her api_token.
fn create(pool: &PgPool, username: &str) -> (User, ApiToken) {
	let connection = pool
		.get()
		.expect("DB pool is expected to be defined in tests. qed.");

	let user = create_user(&connection, username)
		.expect(format!("Create user {} shouldn't error. qed.", username).as_str());
	let api_token = create_api_token(
		&connection,
		format!("{}_subscription_item", username).as_str(),
		&user.id,
	)
	.expect("Create user shouldn't error. qed.");

	(user, api_token)
}

/// Create a random user, with her api_token.
pub fn create_test_user(pool: &PgPool) -> (User, ApiToken) {
	let username = rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(10)
		.collect::<String>();

	create(pool, username.as_str())
}
