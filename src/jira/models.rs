use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraFields,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraFields {
    #[serde(rename = "customfield_10369")]
    pub checklist: Option<ChecklistField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChecklistField {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub version: i32,
    pub content: Vec<ContentNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub content: Option<Vec<ContentNode>>,
    pub attrs: Option<NodeAttributes>,
    pub text: Option<String>,
    pub marks: Option<Vec<Mark>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeAttributes {
    pub level: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mark {
    #[serde(rename = "type")]
    pub mark_type: String,
}

#[derive(Debug, Clone)]
pub struct JiraCredentials {
    pub email: String,
    pub api_token: String,
    pub base_url: String,
}

impl JiraCredentials {
    pub fn from_env() -> Result<Self, crate::error::Error> {
        use std::env;

        let email = env::var("JIRA_USER_EMAIL")
            .map_err(|_| crate::error::Error::EnvVarNotSet("JIRA_USER_EMAIL".to_string()))?;

        let api_token = env::var("JIRA_API_TOKEN")
            .map_err(|_| crate::error::Error::EnvVarNotSet("JIRA_API_TOKEN".to_string()))?;

        let base_url = env::var("JIRA_BASE_URL")
            .map_err(|_| crate::error::Error::EnvVarNotSet("JIRA_BASE_URL".to_string()))?;

        Ok(Self {
            email,
            api_token,
            base_url,
        })
    }
}
