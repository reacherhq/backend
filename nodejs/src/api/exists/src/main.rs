use check_if_email_exists::email_exists;
use http::{header, StatusCode};
use now_lambda::{error::NowError, lambda, IntoResponse, Request, Response};
use serde_json;
use std::{borrow::Cow, collections::HashMap, error::Error};
use url::Url;

fn handler(request: Request) -> Result<impl IntoResponse, NowError> {
	let uri_str = request.uri().to_string();
	let url = Url::parse(&uri_str).unwrap();

	// Create a hash map of query parameters
	let hash_query: HashMap<_, _> = url.query_pairs().to_owned().collect();

	if let Some(ref to_email) = hash_query.get("to_email") {
		let from_email = hash_query
			.get("from_email")
			.unwrap_or(&Cow::Borrowed("user@example.org"));
		Ok(Response::builder()
			.status(StatusCode::OK)
			.header(header::CONTENT_TYPE, "application/json")
			.body(
				serde_json::to_string(&email_exists(to_email, from_email))
					.expect("email_exists gives a serializable output. qed."),
			)
			.expect("Failed to render response"))
	} else {
		Ok(Response::builder()
			.status(StatusCode::UNPROCESSABLE_ENTITY)
			.body("`to_email` is a required query param".to_string())
			.expect("Failed to render response"))
	}
}

// Start the runtime with the handler
fn main() -> Result<(), Box<dyn Error>> {
	Ok(lambda!(handler))
}
