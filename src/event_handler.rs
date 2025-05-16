use crate::error::Error;
use crate::github::models::{extract_issue_keys, PullRequestPayload};
use crate::jira::ChecklistManipulator;
use serde_json::Value;
use tracing::{debug, info};

/// Handles GitHub webhook events
pub async fn handle_event(event_type: &str, payload: &[u8]) -> Result<(), Error> {
    // Only process pull_request events
    if event_type != "pull_request" {
        info!("Ignoring non-pull_request event: {}", event_type);
        return Ok(());
    }

    // Parse the payload
    let payload: PullRequestPayload = serde_json::from_slice(payload)
        .map_err(|e| Error::PayloadParseError(e.to_string()))?;

    info!(
        "Processing pull_request event: action={}, number={}",
        payload.action, payload.pull_request.number
    );

    // Extract issue keys from the PR title
    let current_issue_keys = extract_issue_keys(&payload.pull_request.title);
    debug!("Current issue keys: {:?}", current_issue_keys);

    // For edited events with title changes, extract issue keys from the old title
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

    // Process the issue keys
    process_issue_keys(
        &current_issue_keys,
        &old_issue_keys,
        &payload.pull_request.html_url,
        &payload.action,
    )
    .await
}

/// Processes issue keys extracted from PR titles
async fn process_issue_keys(
    current_keys: &[String],
    old_keys: &[String],
    pr_url: &str,
    action: &str,
) -> Result<(), Error> {
    // Create a checklist manipulator
    let checklist_manipulator = match ChecklistManipulator::from_env() {
        Ok(manipulator) => manipulator,
        Err(e) => {
            info!("Failed to create checklist manipulator: {}", e);
            return Err(e);
        }
    };

    // For each current issue key, add the PR URL to the issue
    for key in current_keys {
        info!("Adding PR link to issue: {}", key);
        if let Err(e) = checklist_manipulator.add_pr_url(key, pr_url).await {
            info!("Failed to add PR link to issue {}: {}", key, e);
            // Continue with other issues even if one fails
        }
    }

    // For title changes, remove PR URL from issues no longer referenced
    if action == "edited" && !old_keys.is_empty() {
        let removed_keys: Vec<&String> = old_keys
            .iter()
            .filter(|key| !current_keys.contains(key))
            .collect();

        for key in removed_keys {
            info!("Removing PR link from issue: {}", key);
            if let Err(e) = checklist_manipulator.remove_pr_url(key, pr_url).await {
                info!("Failed to remove PR link from issue {}: {}", key, e);
                // Continue with other issues even if one fails
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
