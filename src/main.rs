// Reacher
// Copyright (C) 2018-2020 Amaury Martiny

// Reacher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Reacher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Reacher.  If not, see <http://www.gnu.org/licenses/>.

mod handlers;

use std::env;
use warp::Filter;

/// Run a HTTP server using warp.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	// Use an empty string if we don't have any env variable for sentry. Sentry
	// will just silently ignore.
	let _sentry = sentry::init(env::var("RCH_SENTRY_DSN").unwrap_or_else(|_| "".into()));
	// Sentry will also catch panics.
	sentry::integrations::panic::register_panic_handler();

	// POST /check_email
	let routes = warp::post()
		.and(warp::path("check_email"))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handlers::check_email)
		// View access logs by setting `RUST_LOG=reacher`.
		.with(warp::log("reacher"));

	// Since we're running the HTTP server inside a Docker container, we
	// use 0.0.0.0. The port is 8080 as per Fly documentation.
	warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
	Ok(())
}
