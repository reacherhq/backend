use warp::reject;

/// Catch all error struct for the bulk endpoints
#[derive(Debug)]
pub enum BulkError {
	EmptyInput,
	Db(sqlx::Error),
	Csv,
	Json,
}

// Defaults to Internal server error
impl reject::Reject for BulkError {}

// wrap sql errors as db errors for reacher
impl From<sqlx::Error> for BulkError {
	fn from(e: sqlx::Error) -> Self {
		BulkError::Db(e)
	}
}
