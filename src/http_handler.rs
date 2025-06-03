use crate::error::*;
use crate::types::{WebhookEventType, WebhookSecret};
use crate::{github, jira};

pub(crate) async fn function_handler(
    jira_client: jira::JiraClient,
    webhook_secret: WebhookSecret,
    event: lambda_http::Request,
    dry_run: bool,
) -> Result<()> {
    let signature = event
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok());

    use lambda_http::Body;
    let body_bytes = match event.body() {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes,
        Body::Empty => &[],
    };

    github::validate_signature(body_bytes, signature, &webhook_secret)?;

    let event_type = event
        .headers()
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match WebhookEventType::from_str(event_type) {
        WebhookEventType::PullRequest => {
            let payload = serde_json::from_slice(body_bytes)?;
            crate::event_handler::handle_pull_request_event(payload, jira_client, dry_run).await
        }
        WebhookEventType::Other(event_type) => Err(Error::InvalidEventType(event_type)),
    }
}

pub(crate) fn result_to_http_reponse(response: Result<()>) -> LambdaResult {
    let Err(error) = response else {
        return LambdaResult::ok_response();
    };

    tracing::trace!(?error, "function handler error occurred");

    use crate::error::Error::*;
    match error {
        // configuration mistakes, we should never see those here
        BadUrlGenerated(_) | EmptyWebhookSecret | EnvVarNotSet { .. } | EnvVarBadValue { .. } => {
            panic!("Configuration error: {:?}", error);
        }

        // possible configuration mistake on GitHub
        InvalidEventType(event_type) => {
            tracing::warn!("Received event type that was unexpected: {}", event_type);

            // return 200, because technically everything went well
            return LambdaResult::ok_response();
        }

        // request validation errors
        MissingSignatureHeader | InvalidWebhookSignature | PayloadDeserialization(_) => {
            tracing::warn!("Request validation error: {:?}", error);
        }

        // API errors
        JiraApi(_) | HttpClient(_) | AwsKms(_) => {
            tracing::error!("API error: {:?}", error);
        }

        // other kinds of errors
        Internal(_) => {
            tracing::error!("Internal error: {:?}", error);
        }
    };

    LambdaResult::not_ok_response()
}
