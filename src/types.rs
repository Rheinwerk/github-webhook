use crate::error::Error;

#[derive(Clone, Debug)]
pub struct WebhookSecret(String);

impl WebhookSecret {
    pub fn new(secret: String) -> Result<Self, Error> {
        if secret.is_empty() {
            return Err(Error::EmptyWebhookSecret);
        }
        Ok(Self(secret))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WebhookEventType {
    PullRequest,
    Other(String),
}

impl WebhookEventType {
    pub fn from_str(event_type: &str) -> Self {
        match event_type {
            "pull_request" => Self::PullRequest,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn is_pull_request(&self) -> bool {
        matches!(self, Self::PullRequest)
    }
}
