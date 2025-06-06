use lambda_http::{run, service_fn, tracing};

mod config;
mod error;
mod event_handler;
mod github;
mod http_handler;
mod jira;
mod types;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    tracing::init_default_subscriber();
    let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::v2025_01_17()).await;
    let aws_kms = aws_sdk_kms::Client::new(&aws_config);

    let config = config::Config::from_env(&aws_kms).await?;

    let jira_client = jira::JiraClient::new(config.jira_config);

    run(service_fn(move |event| {
        let client = jira_client.clone();
        let webhook_secret = config.webhook_secret.clone();

        async move {
            http_handler::result_to_http_reponse(
                http_handler::function_handler(client, webhook_secret, event, config.dry_run).await,
            )
        }
    }))
    .await
}
