// Reacher - Email Verification
// Copyright (C) 2018-2022 Reacher

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
	routes::{bulk::post::email_verification_task, create_routes},
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
	log::info!(target: "reacher", "Running Reacher v{}", CARGO_PKG_VERSION);

	// Read from .env file if present.
	let _ = dotenv();

	env_logger::init();
	let pg_conn = env::var("DATABASE_URL").unwrap();
	let pg_max_conn = env::var("DATABASE_MAX_CONNECTIONS").map_or(5, |var| {
		var.parse::<u32>()
			.expect("Environment variable DATABASE_MAX_CONNECTIONS should parse to u32")
	});
	let min_task_conc = env::var("MINIMUM_TASK_CONCURRENCY").map_or(10, |var| {
		var.parse::<usize>()
			.expect("Environment variable MINIMUM_TASK_CONCURRENCY should parse to usize")
	});
	let max_conc_task_fetch = env::var("MAXIMUM_CONCURRENT_TASK_FETCH").map_or(20, |var| {
		var.parse::<usize>()
			.expect("Environment variable MAXIMUM_CONCURRENT_TASK_FETCH should parse to usize")
	});

	// create connection pool with database
	// connection pool internally the shared db connection
	// with arc so it can safely be cloned and shared across threads
	let pool = PgPoolOptions::new()
		.max_connections(pg_max_conn)
		.connect(pg_conn.as_str())
		.await?;

	// registry needs to be given list of jobs it can accept
	let registry = JobRegistry::new(&[email_verification_task]);

	// create runner for the message queue associated
	// with this job registry
	#[allow(unused_variables)]
	let registry = registry
		// Create a job runner using the connection pool.
		.runner(&pool)
		// Here is where you can configure the job runner
		// Aim to keep 10-20 jobs running at a time.
		.set_concurrency(min_task_conc, max_conc_task_fetch)
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

	warp::serve(routes).run((host, port)).await;
	Ok(())
}
