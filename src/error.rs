use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Empty webhook secret")]
    EmptyWebhookSecret,

    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,

    #[error("Missing webhook signature header")]
    MissingSignatureHeader,

    #[error("Failed to parse webhook payload: {0}")]
    PayloadParseError(String),

    #[error("GitHub API error: {0}")]
    GitHubApiError(String),

    #[error("Jira API error: {0}")]
    JiraApiError(String),

    #[error("HTTP client error: {0}")]
    HttpClientError(String),

    #[error("Environment variable not set: {0}")]
    EnvVarNotSet(String),

    #[error("Failed to parse Jira field: {0}")]
    JiraFieldParseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}