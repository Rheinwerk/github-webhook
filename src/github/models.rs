use serde::{Deserialize, Serialize};

/// GitHub webhook payload for pull request events
#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub changes: Option<Changes>,
}

/// GitHub pull request
#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
}

/// Changes in a pull request (for edited events)
#[derive(Debug, Deserialize, Serialize)]
pub struct Changes {
    pub title: Option<TitleChange>,
}

/// Title change in a pull request
#[derive(Debug, Deserialize, Serialize)]
pub struct TitleChange {
    pub from: String,
}

/// Extract issue keys from a pull request title
/// 
/// The title format is: [ISSUE-123,ISSUE-234] Description
pub fn extract_issue_keys(title: &str) -> Vec<String> {
    let re = regex::Regex::new(r"^\[([\w\-,\s]+)\]").unwrap();
    
    if let Some(captures) = re.captures(title) {
        if let Some(keys_str) = captures.get(1) {
            return keys_str
                .as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_issue_keys_single() {
        let title = "[ISSUE-123] This is a test PR";
        let keys = extract_issue_keys(title);
        assert_eq!(keys, vec!["ISSUE-123"]);
    }
    
    #[test]
    fn test_extract_issue_keys_multiple() {
        let title = "[ISSUE-123, ISSUE-234] This is a test PR";
        let keys = extract_issue_keys(title);
        assert_eq!(keys, vec!["ISSUE-123", "ISSUE-234"]);
    }
    
    #[test]
    fn test_extract_issue_keys_none() {
        let title = "This is a test PR without issue keys";
        let keys = extract_issue_keys(title);
        assert!(keys.is_empty());
    }
    
    #[test]
    fn test_extract_issue_keys_empty_brackets() {
        let title = "[] This is a test PR with empty brackets";
        let keys = extract_issue_keys(title);
        assert!(keys.is_empty());
    }
}