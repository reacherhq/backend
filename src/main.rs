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

extern crate http;
extern crate lambda_http;
extern crate lambda_runtime;
extern crate serde_json;

use check_if_email_exists::email_exists;
use futures::executor::block_on;
use http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, ORIGIN};
use lambda_http::{lambda, IntoResponse, Request, RequestExt, Response};
use lambda_runtime::{error::HandlerError, Context};
use serde::Serialize;
use std::borrow::Cow;

fn main() {
	lambda!(handler)
}

#[derive(Serialize)]
struct ErrorOutput {
	error: String,
}

fn handler(request: Request, _: Context) -> Result<impl IntoResponse, HandlerError> {
	let query_params = request.query_string_parameters();
	if let Some(to_email) = query_params.get("to_email") {
		let from_email = query_params
			.get("from_email")
			.unwrap_or(&Cow::Borrowed("user@example.org"));

		let response = block_on(email_exists(&to_email, &from_email));
		let serialized = serde_json::to_string(&response).expect("response is serializable. qed.");

		// In case the request header doesn't have ORIGIN, we send back an error
		if !request.headers().contains_key(ORIGIN) {
			let error = ErrorOutput {
				error: "Missing `Origin` header".into(),
			};

			return Ok(Response::builder()
				.status(400)
				.header(CONTENT_TYPE, "application/json")
				.body(serde_json::to_string(&error).expect("`error` is serializable. qed."))
				.expect("`error` is serializable. qed."));
		}

		Ok(Response::builder()
			.status(200)
			.header(CONTENT_TYPE, "application/json")
			.header(ACCESS_CONTROL_ALLOW_ORIGIN, &request.headers()[ORIGIN])
			.body(serialized)
			.expect("`serialized` is serializable. qed."))
	} else {
		let serialized = serde_json::to_string(&ErrorOutput {
			error: "`to_email` is a required query param".into(),
		})
		.expect("ErrorOutput is serializable. qed.");

		Ok(Response::builder()
			.status(422)
			.header(CONTENT_TYPE, "application/json")
			.body(serialized)
			.expect("Failed to render response."))
	}
}
