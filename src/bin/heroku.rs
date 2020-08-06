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

use reacher_backend::{db::connect_db, routes::create_routes, sentry_util::setup_sentry};
use std::{env, net::IpAddr};

/// Run a HTTP server using warp.
///
/// # Panics
///
/// If at least one of the environment variables:
/// - RCH_HTTP_HOST
/// - RCH_PROXY_HOST
/// - RCH_PROXY_PORT
/// is malformed, then the program will panic.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();
	let _guard = setup_sentry();

	let database_url = env::var("RCH_DATABASE_URL").expect("RCH_DATABASE_URL must be set. qed.");
	let pool = connect_db(&database_url);

	let routes = create_routes(pool);

	let host = env::var("RCH_HTTP_HOST")
		.unwrap_or_else(|_| "127.0.0.1".into())
		.parse::<IpAddr>()
		.expect("Env var RCH_HTTP_HOST is malformed.");
	let port = env::var("PORT")
		.map(|port| port.parse::<u16>().expect("Env var PORT is malformed."))
		.unwrap_or(8080);
	log::info!(target: "reacher", "Server is listening on {}:{}.", host, port);

	warp::serve(routes).run((host, port)).await;
	Ok(())
}
