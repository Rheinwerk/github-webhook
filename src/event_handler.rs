use crate::error::Error;
use crate::github::models::{extract_issue_keys, PullRequestPayload};
use crate::jira::{ChecklistManipulator, JiraClient};
use serde_json::Value;
use tracing::{debug, info};

pub async fn handle_event(
    event_type: &str,
    payload: &[u8],
    jira_client: JiraClient,
) -> Result<(), Error> {
    if event_type != "pull_request" {
        info!("Ignoring non-pull_request event: {}", event_type);
        return Ok(());
    }

    let payload: PullRequestPayload =
        serde_json::from_slice(payload).map_err(|e| Error::PayloadParseError(e.to_string()))?;

    info!(
        "Processing pull_request event: action={}, number={}",
        payload.action, payload.pull_request.number
    );

    let current_issue_keys = extract_issue_keys(&payload.pull_request.title);
    debug!("Current issue keys: {:?}", current_issue_keys);

    let old_issue_keys = if payload.action == "edited" {
        if let Some(changes) = &payload.changes {
            if let Some(title_change) = &changes.title {
                let keys = extract_issue_keys(&title_change.from);
                debug!("Old issue keys: {:?}", keys);
                keys
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    process_issue_keys(
        &current_issue_keys,
        &old_issue_keys,
        &payload.pull_request.html_url,
        &payload.action,
        jira_client,
    )
    .await
}

async fn process_issue_keys(
    current_keys: &[String],
    old_keys: &[String],
    pr_url: &str,
    action: &str,
    jira_client: JiraClient,
) -> Result<(), Error> {
    let checklist_manipulator = ChecklistManipulator::new(&jira_client);

    for key in current_keys {
        info!("Adding PR link to issue: {}", key);
        if let Err(e) = checklist_manipulator.add_pr_url(key, pr_url).await {
            info!("Failed to add PR link to issue {}: {}", key, e);
        }
    }

    if action == "edited" && !old_keys.is_empty() {
        let removed_keys: Vec<&String> = old_keys
            .iter()
            .filter(|key| !current_keys.contains(key))
            .collect();

        for key in removed_keys {
            info!("Removing PR link from issue: {}", key);
            if let Err(e) = checklist_manipulator.remove_pr_url(key, pr_url).await {
                info!("Failed to remove PR link from issue {}: {}", key, e);
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

        let payload_json = serde_json::to_vec(&payload).unwrap();
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
