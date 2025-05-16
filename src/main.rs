use lambda_http::{run, service_fn, tracing, Error};

mod error;
mod event_handler;
mod github;
mod http_handler;
mod types;

use http_handler::function_handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
