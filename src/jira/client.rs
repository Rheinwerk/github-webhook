use crate::error::Error;
use crate::jira::models::{ChecklistField, JiraCredentials, JiraIssue};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct JiraClient {
    client: reqwest::Client,
    credentials: JiraCredentials,
}

impl JiraClient {
    pub fn new(credentials: JiraCredentials) -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials,
        }
    }

    pub fn from_env() -> Result<Self, Error> {
        let credentials = JiraCredentials::from_env()?;
        Ok(Self::new(credentials))
    }

    fn create_headers(&self) -> Result<HeaderMap, Error> {
        use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};

        let mut headers = HeaderMap::new();

        let auth = format!("{}:{}", self.credentials.email, self.credentials.api_token);
        let auth_header = format!("Basic {}", Base64.encode(auth));

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_header)
                .map_err(|e| Error::JiraApiError(format!("Failed to create auth header: {}", e)))?,
        );

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue, Error> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=customfield_10369",
            self.credentials.base_url, issue_key
        );

        debug!("Fetching Jira issue: {}", issue_key);

        let headers = self.create_headers()?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| Error::HttpClientError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());

            return Err(Error::JiraApiError(format!(
                "Failed to get issue {}: {} - {}",
                issue_key, status, body
            )));
        }

        let issue: JiraIssue = response
            .json()
            .await
            .map_err(|e| Error::JiraApiError(format!("Failed to parse issue: {}", e)))?;

        Ok(issue)
    }

    pub async fn update_checklist(
        &self,
        issue_key: &str,
        checklist: &ChecklistField,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/rest/api/3/issue/{}",
            self.credentials.base_url, issue_key
        );

        info!("Updating checklist for issue: {}", issue_key);

        let headers = self.create_headers()?;

        let payload = serde_json::json!({
            "fields": {
                "customfield_10369": checklist
            }
        });

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::HttpClientError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());

            return Err(Error::JiraApiError(format!(
                "Failed to update issue {}: {} - {}",
                issue_key, status, body
            )));
        }

        Ok(())
    }
}
