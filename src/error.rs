use thiserror::Error;

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
}
