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

mod http;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    // Since we're running the HTTP server inside a Docker container, we
    // use 0.0.0.0. Allow for overriding via env variable.
    let http_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    // http_port is, in this order:
    // - the value of `--http-port` flag
    // - if not set, then the $PORT env varialbe
    // - if not set, then 8080
    let http_port = env::var("PORT").unwrap_or_else(|_| "8080".into());

    http::run(&http_host, http_port.parse::<u16>().unwrap()).await?;

    Ok(())
}
