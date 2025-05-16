use crate::error::Error as AppError;
use crate::event_handler::handle_event;
use crate::github::validate_signature;
use crate::jira::JiraClient;
use crate::types::{WebhookEventType, WebhookSecret};
use lambda_http::{Body, Error, Request, Response};
use std::env;
use tracing::{error, info};

/// Main handler for the Lambda function
pub(crate) async fn function_handler(event: Request, jira_client: JiraClient) -> Result<Response<Body>, Error> {
    // Get the webhook secret from environment variables
    let webhook_secret = match get_webhook_secret() {
        Ok(secret) => secret,
        Err(e) => {
            error!("Failed to get webhook secret: {}", e);
            return Ok(create_error_response(500, "Internal server error"));
        }
    };

    // Get the event type from the X-GitHub-Event header
    let event_type = event
        .headers()
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Get the signature from the X-Hub-Signature-256 header
    let signature = event
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok());

    // Get the request body
    let body = event.body();
    let body_bytes = match body {
        Body::Text(text) => text.as_bytes(),
        Body::Binary(bytes) => bytes,
        Body::Empty => &[],
    };

    // Validate the signature
    if let Err(e) = validate_signature(body_bytes, signature, &webhook_secret) {
        error!("Signature validation failed: {}", e);
        return Ok(create_error_response(401, "Invalid signature"));
    }

    // Create webhook event type
    let webhook_event = WebhookEventType::from_str(event_type);

    // Return 200 for non-pull_request events
    if !webhook_event.is_pull_request() {
        info!("Ignoring non-pull_request event: {}", event_type);
        return Ok(create_success_response());
    }

    // Handle the event
    match handle_event(event_type, body_bytes, jira_client).await {
        Ok(_) => {
            info!("Successfully processed event");
            Ok(create_success_response())
        }
        Err(e) => {
            error!("Failed to process event: {}", e);
            Ok(create_error_response(500, "Failed to process event"))
        }
    }
}

/// Get the webhook secret from environment variables
fn get_webhook_secret() -> Result<WebhookSecret, AppError> {
    let secret = env::var("WEBHOOK_SECRET")
        .map_err(|_| AppError::EnvVarNotSet("WEBHOOK_SECRET".to_string()))?;
    WebhookSecret::new(secret)
}

/// Create a success response
fn create_success_response() -> Response<Body> {
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(r#"{"status":"success"}"#.into())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body("Internal server error".into())
                .unwrap()
        })
}

/// Create an error response
fn create_error_response(status: u16, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(format!(r#"{{"status":"error","message":"{}"}}"#, message).into())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body("Internal server error".into())
                .unwrap()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::http::HeaderMap;
    use std::collections::HashMap;

    #[test]
    fn test_create_success_response() {
        let response = create_success_response();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();
        assert_eq!(body_string, r#"{"status":"success"}"#);
    }

    #[test]
    fn test_create_error_response() {
        let response = create_error_response(400, "Bad request");
        assert_eq!(response.status(), 400);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();
        assert_eq!(body_string, r#"{"status":"error","message":"Bad request"}"#);
    }
}
