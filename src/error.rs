use thiserror::Error;

pub type LambdaResult = std::result::Result<lambda_http::Response<lambda_http::Body>, LambdaError>;

pub trait LambdaResultExt {
    fn ok_response() -> LambdaResult {
        use lambda_http::{http::StatusCode, Body, Response};

        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::Empty)
            .expect("Response body can be built"))
    }

    fn not_ok_response() -> LambdaResult {
        use lambda_http::{http::StatusCode, Body, Response};

        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::Empty)
            .expect("Response body can be built"))
    }
}
impl LambdaResultExt for LambdaResult {}

pub type LambdaError = lambda_http::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Empty webhook secret")]
    EmptyWebhookSecret,

    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,

    #[error("Missing webhook signature header")]
    MissingSignatureHeader,

    #[error("Failed to deserialize payload")]
    PayloadDeserialization(#[from] serde_json::Error),

    #[error("Jira API error: {0}")]
    JiraApi(String),

    #[error("Failed to generate url for request")]
    BadUrlGenerated(#[from] url::ParseError),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Environment variable not set: {env_var_name}")]
    EnvVarNotSet { env_var_name: &'static str },

    #[error("Environment variable {env_var_name} has bad value")]
    EnvVarBadValue { env_var_name: &'static str },

    #[error("AWS KMS error: {0:?}")]
    AwsKms(#[from] aws_sdk_kms::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid event type {0}")]
    InvalidEventType(String),
}
