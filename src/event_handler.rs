use crate::error::Error;
use crate::github::models::{extract_issue_key, PullRequest, PullRequestPayload};
use crate::jira::{ChecklistManipulator, JiraClient, PrStatus};

#[tracing::instrument(skip_all,fields(action = %payload.action, pull_request = %payload.pull_request.number))]
pub async fn handle_pull_request_event(
    payload: PullRequestPayload,
    jira_client: JiraClient,
    dry_run: bool,
) -> Result<(), Error> {
    tracing::info!("Processing pull_request event");

    if let Some(issue_key) = extract_issue_key(&payload.pull_request.title) {
        let status = pr_status(&payload.pull_request);
        update_issue(
            &jira_client,
            &issue_key,
            &payload.pull_request.html_url,
            status,
            dry_run,
        )
        .await?
    }

    Ok(())
}

fn pr_status(pr: &PullRequest) -> PrStatus {
    if pr.merged {
        PrStatus::Merged
    } else if pr.state == "closed" {
        PrStatus::Closed
    } else {
        PrStatus::Open
    }
}

#[tracing::instrument(skip(jira_client, issue_key, html_url))]
async fn update_issue(
    jira_client: &JiraClient,
    issue_key: &str,
    html_url: &str,
    status: PrStatus,
    dry_run: bool,
) -> Result<(), Error> {
    tracing::info!("Updating issue");

    let issue = jira_client.get_issue(issue_key).await?;

    let Some(checklist_text) = issue.fields.checklist.text() else {
        tracing::warn!("No checklist found for {issue_key}. Skip update.");
        return Ok(());
    };

    let mut checklist = ChecklistManipulator::new(&checklist_text);

    if !checklist.upsert_pr(html_url, status) {
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
    fn test_extract_issue_key_from_payload() {
        let payload = PullRequestPayload {
            action: "opened".to_string(),
            pull_request: PullRequest {
                title: "[ISSUE-123] Test PR".to_string(),
                html_url: "https://github.com/org/repo/pull/1".to_string(),
                number: 1,
                state: "open".to_string(),
                merged: false,
            },
            changes: None,
        };

        let key = extract_issue_key(&payload.pull_request.title);
        assert_eq!(key, Some("ISSUE-123".to_string()));
    }

    #[test]
    fn test_extract_old_issue_key_from_edited_payload() {
        let payload = PullRequestPayload {
            action: "edited".to_string(),
            pull_request: PullRequest {
                title: "[ISSUE-234] Updated PR".to_string(),
                html_url: "https://github.com/org/repo/pull/1".to_string(),
                number: 1,
                state: "open".to_string(),
                merged: false,
            },
            changes: Some(Changes {
                title: Some(TitleChange {
                    from: "[ISSUE-123] Original PR".to_string(),
                }),
            }),
        };

        let current_key = extract_issue_key(&payload.pull_request.title);
        assert_eq!(current_key, Some("ISSUE-234".to_string()));

        if let Some(changes) = &payload.changes {
            if let Some(title_change) = &changes.title {
                let old_key = extract_issue_key(&title_change.from);
                assert_eq!(old_key, Some("ISSUE-123".to_string()));
            }
        }
    }

    #[test]
    fn test_pr_status_open() {
        let pr = PullRequest {
            title: "Test".to_string(),
            html_url: "https://github.com/org/repo/pull/1".to_string(),
            number: 1,
            state: "open".to_string(),
            merged: false,
        };
        assert_eq!(pr_status(&pr), PrStatus::Open);
    }

    #[test]
    fn test_pr_status_merged() {
        let pr = PullRequest {
            title: "Test".to_string(),
            html_url: "https://github.com/org/repo/pull/1".to_string(),
            number: 1,
            state: "closed".to_string(),
            merged: true,
        };
        assert_eq!(pr_status(&pr), PrStatus::Merged);
    }

    #[test]
    fn test_pr_status_closed() {
        let pr = PullRequest {
            title: "Test".to_string(),
            html_url: "https://github.com/org/repo/pull/1".to_string(),
            number: 1,
            state: "closed".to_string(),
            merged: false,
        };
        assert_eq!(pr_status(&pr), PrStatus::Closed);
    }
}
