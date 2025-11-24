use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraFields,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JiraFields {
    #[serde(rename = "customfield_10369")]
    pub checklist: ContentNode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentNode {
    Doc {
        content: Vec<ContentNode>,
        version: i32,
    },
    Paragraph {
        content: Vec<ContentNode>,
    },
    Text {
        text: String,
    },
    HardBreak,
}

impl ContentNode {
    /// This is the format used by the checklists custom field
    pub fn new_doc_paragraph_text(text: String) -> Self {
        Self::Doc {
            content: vec![Self::Paragraph {
                content: vec![Self::Text { text }],
            }],
            version: 1,
        }
    }

    pub fn text(&self) -> Option<String> {
        match self {
            Self::Text { text } => Some(text.clone()),
            Self::HardBreak => Some("\n".to_string()),
            Self::Doc { content, .. } | Self::Paragraph { content } => {
                let parts: Vec<String> = content.iter()
                    .filter_map(|node| node.text())
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(""))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct JiraConfig {
    pub email: String,
    pub api_token: String,
    pub base_url: reqwest::Url,
}

#[cfg(test)]
mod test {
    use super::JiraIssue;

    fn sample_issue() -> JiraIssue {
        let json = r##"{"expand":"renderedFields,names,schema,operations,editmeta,changelog,versionedRepresentations","id":"1337","self":"https://example.atlassian.net/rest/api/3/issue/1337","key":"TEST-7","fields":{"customfield_10369":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"# Development Process\n-! Task 1: Detail Planning including Risk Assessment\n-! Task 2: Document Risk Assessment\n-! Task 3: Create Feature Branch\n-! Task 4: Development including Tests and Documentation\n-! Task 5: Create Pull Request and Request Review\n-! Task 6: Request Functional Acceptance\n-! Task 7: Prepare Deployment\n-! Task 8: Request Deployment/Merge Approval\n-! Task 9: Merge Pull Request\n-! Task 10: Execute Deployment\n-! Task 11: Verify Production Delivery\n# Pull Requests\n"}]}]}}}"##;
        serde_json::from_str(json).expect("failed to deserialize")
    }

    #[test]
    fn deserializes() {
        sample_issue();
    }
}
