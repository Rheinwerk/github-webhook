use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequestPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub changes: Option<Changes>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Changes {
    pub title: Option<TitleChange>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TitleChange {
    pub from: String,
}

pub fn extract_issue_key(title: &str) -> Option<String> {
    let re = regex::Regex::new(r"^\[?([A-Za-z]+)[\- ]*([0-9]+)").unwrap();

    re.captures(title)
        .and_then(|captures| {
            let prefix = captures.get(1)?.as_str().to_uppercase();
            let number = captures.get(2)?.as_str();
            Some(format!("{}-{}", prefix, number))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_issue_key_single() {
        let title = "[ISSUE-123] This is a test PR";
        let key = extract_issue_key(title);
        assert_eq!(key, Some("ISSUE-123".to_string()));
    }


    #[test]
    fn test_extract_issue_key_none() {
        let title = "This is a test PR without issue keys";
        let key = extract_issue_key(title);
        assert!(key.is_none());
    }

    #[test]
    fn test_extract_issue_key_empty_brackets() {
        let title = "[] This is a test PR with empty brackets";
        let key = extract_issue_key(title);
        assert!(key.is_none());
    }

    #[test]
    fn test_extract_issue_key_space_separated() {
        let title = "Issue 51 - Fix authentication issue";
        let key = extract_issue_key(title);
        assert_eq!(key, Some("ISSUE-51".to_string()));
    }

    #[test]
    fn test_extract_issue_key_hyphen_separated() {
        let title = "Issue-51 Update user interface";
        let key = extract_issue_key(title);
        assert_eq!(key, Some("ISSUE-51".to_string()));
    }

    #[test]
    fn test_extract_issue_key_bracketed_with_hyphen() {
        let title = "[Issue-51] Implement new feature";
        let key = extract_issue_key(title);
        assert_eq!(key, Some("ISSUE-51".to_string()));
    }
}
