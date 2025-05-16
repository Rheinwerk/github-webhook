use crate::error::Error;

/// Newtype for webhook secret to ensure it's not empty
#[derive(Clone, Debug)]
pub struct WebhookSecret(String);

impl WebhookSecret {
    /// Creates a new WebhookSecret, ensuring it's not empty
    pub fn new(secret: String) -> Result<Self, Error> {
        if secret.is_empty() {
            return Err(Error::EmptyWebhookSecret);
        }
        Ok(Self(secret))
    }

    /// Returns the inner secret value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// GitHub webhook event types
#[derive(Debug, PartialEq, Eq)]
pub enum WebhookEventType {
    PullRequest,
    Other(String),
}

impl WebhookEventType {
    /// Creates a new WebhookEventType from a string
    pub fn from_str(event_type: &str) -> Self {
        match event_type {
            "pull_request" => Self::PullRequest,
            other => Self::Other(other.to_string()),
        }
    }

    /// Returns true if this is a pull request event
    pub fn is_pull_request(&self) -> bool {
        matches!(self, Self::PullRequest)
    }
}