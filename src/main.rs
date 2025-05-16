#![deny(unused)]
use lambda_http::{run, service_fn, tracing, Error};

mod error;
mod event_handler;
mod github;
mod http_handler;
mod jira;
mod types;

use http_handler::function_handler;
use jira::JiraClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let jira_client = match JiraClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create JiraClient: {}", e);
            panic!("Failed to create JiraClient: {}", e);
        }
    };

    run(service_fn(move |event| {
        let client = jira_client.clone();
        function_handler(event, client)
    }))
    .await
}
