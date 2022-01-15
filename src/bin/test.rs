use sqlx::postgres::PgPoolOptions;

use core::time;
use std::{env, error::Error, thread};

use dotenv::dotenv;
use sqlxmq::{job, CurrentJob, JobRegistry};

// Arguments to the `#[job]` attribute allow setting default job options.
#[job(channel_name = "foo")]
async fn example_job(
	// The first argument should always be the current job.
	mut current_job: CurrentJob,
	// Additional arguments are optional, but can be used to access context
	// provided via [`JobRegistry::set_context`].
	message: &'static str,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
	// Decode a JSON payload
	let who: Option<String> = current_job.json()?;

	// Do some work
	println!("{}, {}!", message, who.as_deref().unwrap_or("world"));

	// Mark the job as complete
	current_job.complete().await?;

	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	dotenv().expect(".env file with PGPASSWORD variable");
	let pg_password = env::var("PGPASSWORD").unwrap();
	let connection_uri = format!("postgres://postgres:{}@localhost/postgres", pg_password);

	// create connection pool with database
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(connection_uri.as_str())
		.await?;

	// Construct a job registry from our single job.
	let mut registry = JobRegistry::new(&[example_job]);
	// Here is where you can configure the registry
	// registry.set_error_handler(...)

	// And add context
	registry.set_context("Hello");

	let runner = registry
		// Create a job runner using the connection pool.
		.runner(&pool)
		// Here is where you can configure the job runner
		// Aim to keep 10-20 jobs running at a time.
		.set_concurrency(10, 20)
		// Start the job runner in the background.
		.run()
		.await?;

	let response = example_job
		.builder()
		// This is where we can override job configuration
		.set_channel_name("bar")
		.set_json("John")?
		.spawn(&pool)
		.await?;

	dbg!(response);

	let response = example_job
		.builder()
		// This is where we can override job configuration
		.set_channel_name("bar")
		.set_json("John")?
		.spawn(&pool)
		.await?;

	dbg!(response);

	// allow jobs to complete
	thread::sleep(time::Duration::from_secs(5));

	// The job runner will continue listening and running
	// jobs until `runner` is dropped.
	Ok(())
}
