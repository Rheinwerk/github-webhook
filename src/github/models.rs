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

pub fn extract_issue_keys(title: &str) -> Vec<&str> {
    let re = regex::Regex::new(r"^\[([\w\-,\s]+)]").unwrap();

    re.captures(title)
        .and_then(|captures| captures.get(1))
        .map(|keys_str| {
            keys_str
                .as_str()
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
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
