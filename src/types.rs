use crate::error::Error;

#[derive(Clone, Debug)]
pub struct WebhookSecret(Vec<u8>);

impl WebhookSecret {
    pub fn new(secret: impl Into<Vec<u8>>) -> Result<Self, Error> {
        let secret = secret.into();
        if secret.is_empty() {
            return Err(Error::EmptyWebhookSecret);
        }
        Ok(Self(secret))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
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
