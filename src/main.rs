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
use std::net::SocketAddr;
use warp::Filter;

/// Run a HTTP server using warp.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	let cors = warp::cors()
		.allow_origins(vec!["http://127.0.0.1:3000", "https://reacherhq.github.io"])
		.allow_headers(vec!["*"])
		.allow_methods(vec!["POST"]);

	// POST / {"to_email":""}
	let routes = warp::post()
		.and(warp::path("check_email"))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handlers::check_email)
		.with(cors);

	// Since we're running the HTTP server inside a Docker container, we
	// use 0.0.0.0. Allow for overriding via env variable.
	let http_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
	// http_port is, in this order:
	// - the $PORT env varialbe
	// - if not set, then 8080
	let http_port = env::var("PORT").unwrap_or_else(|_| "8080".into());
	let addr = SocketAddr::new(http_host.parse()?, http_port.parse()?);

	warp::serve(routes).run(addr).await;
	Ok(())
}
