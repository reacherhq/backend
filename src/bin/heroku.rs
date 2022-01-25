// Reacher - Email Verification
// Copyright (C) 2018-2021 Reacher

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

use reacher_backend::{
	routes::{create_routes, manage_job::post::email_verification_task},
	sentry_util::{setup_sentry, CARGO_PKG_VERSION},
};

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlxmq::JobRegistry;
use std::{env, net::IpAddr};

/// Run a HTTP server using warp.
///
/// # Panics
///
/// The program panics if at least one of the environment variables is
/// malformed:
/// - RCH_HTTP_HOST,
/// - PORT.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	dotenv().expect(".env file with DATABASE_URL variable");

	env_logger::init();
	let pg_conn = env::var("DATABASE_URL").unwrap();

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(pg_conn.as_str())
		.await?;

	// registry needs to be given list of jobs it can accept
	let mut registry = JobRegistry::new(&[email_verification_task]);
	registry.set_context("Hello");

	// create runner for the message queue associated
	// with this job registry
	let registry = registry
		// Create a job runner using the connection pool.
		.runner(&pool)
		// Here is where you can configure the job runner
		// Aim to keep 10-20 jobs running at a time.
		.set_concurrency(10, 20)
		// Start the job runner in the background.
		.run()
		.await?;

	// Setup warp server
	let _guard = setup_sentry();

	let routes = create_routes(pool);

	let host = env::var("RCH_HTTP_HOST")
		.unwrap_or_else(|_| "127.0.0.1".into())
		.parse::<IpAddr>()
		.expect("Environment variable RCH_HTTP_HOST is malformed.");
	let port = env::var("PORT")
		.map(|port| {
			port.parse::<u16>()
				.expect("Environment variable PORT is malformed.")
		})
		.unwrap_or(8080);
	log::info!(target: "reacher", "Server is listening on {}:{}.", host, port);

	log::info!(target: "reacher", "Running Reacher v{}", CARGO_PKG_VERSION);
	warp::serve(routes).run((host, port)).await;
	Ok(())
}
