use crate::error::Error;
use crate::jira::models::{ContentNode, JiraConfig, JiraIssue};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

#[derive(Debug, Clone)]
pub struct JiraClient {
    client: reqwest::Client,
    config: JiraConfig,
}

impl JiraClient {
    pub fn new(credentials: JiraConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config: credentials,
        }
    }

    fn create_headers(&self) -> Result<HeaderMap, Error> {
        use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};

        let mut headers = HeaderMap::new();

        let auth = format!("{}:{}", self.config.email, self.config.api_token);
        let auth_header = format!("Basic {}", Base64.encode(auth));

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_header)
                .map_err(|_| Error::JiraApi("Failed to create auth header".to_string()))?,
        );

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue, Error> {
        let url = self.config.base_url.join(&format!(
            "rest/api/3/issue/{}?fields=customfield_10369",
            issue_key
        ))?;

        tracing::debug!("Fetching Jira issue: {}", issue_key);

        let headers = self.create_headers()?;

        let response = self.client.get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            let status = response.status();

            return Err(Error::JiraApi(format!(
                "Failed to get issue {issue_key}: {status}",
            )));
        }

        let issue: JiraIssue = response
            .json()
            .await
            .map_err(|e| Error::JiraApi(format!("Failed to parse issue: {}", e)))?;

        Ok(issue)
    }

    pub async fn update_checklist(
        &self,
        issue_key: &str,
        checklist: impl ToString,
    ) -> Result<(), Error> {
        let url = self
            .config
            .base_url
            .join(&format!("rest/api/3/issue/{}", issue_key))?;

        tracing::info!("Updating checklist for issue: {}", issue_key);

        let headers = self.create_headers()?;

        let payload = serde_json::json!({
            "fields": {
                "customfield_10369": ContentNode::new_doc_paragraph_text(checklist.to_string())
            }
        });

        self.client
            .put(url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
