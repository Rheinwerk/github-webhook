use crate::error::Error;
use crate::github::models::{extract_issue_keys, PullRequestPayload};
use crate::jira::{ChecklistManipulator, JiraClient};

#[tracing::instrument(skip_all,fields(action = %payload.action, pull_request = %payload.pull_request.number))]
pub async fn handle_pull_request_event(
    payload: PullRequestPayload,
    jira_client: JiraClient,
    dry_run: bool,
) -> Result<(), Error> {
    tracing::info!("Processing pull_request event");

    for issue_key in extract_issue_keys(&payload.pull_request.title) {
        update_issue(
            &jira_client,
            issue_key,
            &payload.pull_request.html_url,
            dry_run,
        )
        .await?
    }

    Ok(())
}

#[tracing::instrument(skip(jira_client, issue_key, html_url))]
async fn update_issue(
    jira_client: &JiraClient,
    issue_key: &str,
    html_url: &str,
    dry_run: bool,
) -> Result<(), Error> {
    tracing::info!("Updating issue");

    let issue = jira_client.get_issue(issue_key).await?;

    let Some(checklist) = &issue.fields.checklist.text() else {
        tracing::warn!("No checklist found for {issue_key}. Skip update.");
        return Ok(());
    };

    let mut checklist = ChecklistManipulator::new(checklist);

    if !checklist.push_pr(html_url) {
        tracing::debug!("checklist not updated, skip");
        return Ok(());
    }

    if dry_run {
        tracing::info!("dry run mode. would have updated issue")
    } else {
        tracing::debug!("Updating checklist");
        jira_client
            .update_checklist(issue_key, checklist.to_string())
            .await?
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
