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

use env_logger;
use log;
use std::fmt;
use warp::Filter;

/// Display an Option<T> nicely.
struct OptFmt<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptFmt<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(ref t) = self.0 {
			fmt::Display::fmt(t, f)
		} else {
			f.write_str("-")
		}
	}
}

/// Run a HTTP server using warp.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	env_logger::init();

	// Custom logger
	let log = warp::log::custom(|info| {
		// TODO?
		// - response content length?
		log::info!(
			target: "reacher",
			"\"{} {}\" {} \"{}\" \"{}\" \"{}\" {:?}",
			info.method(),
			info.path(),
			info.status().as_u16(),
			OptFmt(info.referer()),
			OptFmt(info.user_agent()),
			OptFmt(info.host()),
			info.elapsed(),
		);
	});

	let cors = warp::cors()
		// FIXME Is there a way to allow all headers?
		// https://github.com/seanmonstar/warp/issues/536
		.allow_headers(vec![
			"Accept",
			"Accept-Encoding",
			"Accept-Language",
			"Access-Control-Request-Method",
			"Access-Control-Request-Headers",
			"Connection",
			"Content-Type",
			"DNT",
			"Host",
			"Origin",
			"Referer",
			"Sec-Fetch-Dest",
			"Sec-Fetch-Mode",
			"Sec-Fetch-Site",
			"User-Agent",
		])
		.allow_origins(vec!["http://localhost:8000", "https://reacherhq.github.io"])
		.allow_methods(vec!["OPTIONS", "POST"]);

	// POST /check_email
	let routes = warp::post()
		.and(warp::path("check_email"))
		// When accepting a body, we want a JSON body (and to reject huge
		// payloads)...
		.and(warp::body::content_length_limit(1024 * 16))
		.and(warp::body::json())
		.and_then(handlers::check_email)
		.with(log)
		.with(cors);

	// Since we're running the HTTP server inside a Docker container, we
	// use 0.0.0.0. The port is 8080 as per Fly documentation.
	warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
	Ok(())
}
