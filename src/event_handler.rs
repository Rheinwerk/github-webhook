use crate::error::Error;
use crate::github::models::{extract_issue_keys, PullRequestPayload};
use crate::jira::{ChecklistManipulator, JiraClient};

pub async fn handle_event(
    event_type: &str,
    payload: &[u8],
    jira_client: JiraClient,
    dry_run: bool,
) -> Result<(), Error> {
    if event_type != "pull_request" {
        tracing::info!("Ignoring non-pull_request event: {}", event_type);
        return Ok(());
    }

    let payload: PullRequestPayload = serde_json::from_slice(payload)?;

    tracing::info!(
        payload.action,
        payload.pull_request.number,
        "Processing pull_request event",
    );

    let current_issue_keys = extract_issue_keys(&payload.pull_request.title);

    for issue_key in current_issue_keys {
        let issue = jira_client.get_issue(issue_key).await?;
        if let Some(checklist) = &issue.fields.checklist.text() {
            let mut cm = ChecklistManipulator::new(checklist);
            if cm.push_pr(&payload.pull_request.html_url) {
                if dry_run {
                    tracing::info!(
                        issue_key,
                        pull_request = payload.pull_request.html_url,
                        "dry run mode. would have updated issue"
                    )
                } else {
                    jira_client
                        .update_checklist(issue_key, cm.to_string())
                        .await?
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::models::{Changes, PullRequest, TitleChange};

    #[test]
    fn test_extract_issue_keys_from_payload() {
        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: PullRequest {
                title: "[ISSUE-123, ISSUE-234] Test PR".to_string(),
                html_url: "https://github.com/org/repo/pull/1".to_string(),
                number: 1,
                state: "open".to_string(),
            },
            changes: None,
        };

        let keys = extract_issue_keys(&payload.pull_request.title);
        assert_eq!(keys, vec!["ISSUE-123", "ISSUE-234"]);
    }

    #[test]
    fn test_extract_old_issue_keys_from_edited_payload() {
        let payload = PullRequestPayload {
            action: "edited".to_string(),
            pull_request: PullRequest {
                title: "[ISSUE-234] Updated PR".to_string(),
                html_url: "https://github.com/org/repo/pull/1".to_string(),
                number: 1,
                state: "open".to_string(),
            },
            changes: Some(Changes {
                title: Some(TitleChange {
                    from: "[ISSUE-123] Original PR".to_string(),
                }),
            }),
        };

        let current_keys = extract_issue_keys(&payload.pull_request.title);
        assert_eq!(current_keys, vec!["ISSUE-234"]);

        if let Some(changes) = &payload.changes {
            if let Some(title_change) = &changes.title {
                let old_keys = extract_issue_keys(&title_change.from);
                assert_eq!(old_keys, vec!["ISSUE-123"]);
            }
        }
    }
}
