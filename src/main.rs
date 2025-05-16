#![deny(clippy::pedantic)]
use lambda_http::{run, service_fn, tracing, Error};

mod config;
mod error;
mod event_handler;
mod github;
mod http_handler;
mod jira;
mod types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let config = config::Config::from_env()?;

    let jira_client = jira::JiraClient::new(config.jira_config);

    run(service_fn(move |event| {
        let client = jira_client.clone();
        let webhook_secret = config.webhook_secret.clone();

        http_handler::function_handler(event, config.dry_run, webhook_secret, client)
    }))
    .await
}
