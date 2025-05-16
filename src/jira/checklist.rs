use crate::error::Error;
use crate::jira::models::{ChecklistField, ContentNode, Mark, NodeAttributes};
use crate::jira::JiraClient;
use tracing::{debug, info};

pub struct ChecklistManipulator<'a> {
    client: &'a JiraClient,
}

impl<'a> ChecklistManipulator<'a> {
    pub fn new(client: &'a JiraClient) -> Self {
        Self { client }
    }

    pub async fn add_pr_url(&self, issue_key: &str, pr_url: &str) -> Result<(), Error> {
        info!("Adding PR URL to issue {}: {}", issue_key, pr_url);

        let issue = self.client.get_issue(issue_key).await?;

        let mut checklist = match issue.fields.checklist {
            Some(checklist) => checklist,
            None => create_empty_checklist(),
        };

        let pr_section_index = find_or_create_pr_section(&mut checklist);

        if pr_url_exists(&checklist, pr_section_index, pr_url) {
            debug!("PR URL already exists in issue {}", issue_key);
            return Ok(());
        }

        add_pr_url_to_section(&mut checklist, pr_section_index, pr_url);

        self.client.update_checklist(issue_key, &checklist).await
    }

    pub async fn remove_pr_url(&self, issue_key: &str, pr_url: &str) -> Result<(), Error> {
        info!("Removing PR URL from issue {}: {}", issue_key, pr_url);

        let issue = self.client.get_issue(issue_key).await?;

        let Some(mut checklist) = issue.fields.checklist else {
            debug!("Issue {} has no checklist field", issue_key);
            return Ok(());
        };

        let Some(pr_section_index) = find_pr_section(&checklist) else {
            debug!("Issue {} has no Pull Requests section", issue_key);
            return Ok(());
        };

        if !pr_url_exists(&checklist, pr_section_index, pr_url) {
            debug!("PR URL does not exist in issue {}", issue_key);
            return Ok(());
        }

        remove_pr_url_from_section(&mut checklist, pr_section_index, pr_url);

        self.client.update_checklist(issue_key, &checklist).await
    }
}

fn create_empty_checklist() -> ChecklistField {
    ChecklistField {
        doc_type: "doc".to_string(),
        version: 1,
        content: Vec::new(),
    }
}

fn find_pr_section(checklist: &ChecklistField) -> Option<usize> {
    checklist.content.iter().position(|node| {
        node.node_type == "heading"
            && node.content.as_ref().map_or(false, |content| {
                content.iter().any(|text_node| {
                    text_node.node_type == "text"
                        && text_node
                            .text
                            .as_ref()
                            .map_or(false, |text| text.trim() == "Pull Requests")
                })
            })
    })
}

fn find_or_create_pr_section(checklist: &mut ChecklistField) -> usize {
    if let Some(index) = find_pr_section(checklist) {
        return index;
    }

    let heading = ContentNode {
        node_type: "heading".to_string(),
        attrs: Some(NodeAttributes { level: Some(2) }),
        content: Some(vec![ContentNode {
            node_type: "text".to_string(),
            text: Some("Pull Requests".to_string()),
            content: None,
            attrs: None,
            marks: None,
        }]),
        text: None,
        marks: None,
    };

    checklist.content.push(heading);

    checklist.content.len() - 1
}

fn pr_url_exists(checklist: &ChecklistField, section_index: usize, pr_url: &str) -> bool {
    for i in (section_index + 1)..checklist.content.len() {
        let node = &checklist.content[i];

        if node.node_type == "heading" {
            break;
        }

        if node.node_type == "paragraph" {
            if let Some(content) = &node.content {
                for text_node in content {
                    if text_node.node_type == "text" {
                        if let Some(text) = &text_node.text {
                            if text.contains(pr_url) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn add_pr_url_to_section(checklist: &mut ChecklistField, section_index: usize, pr_url: &str) {
    let paragraph = ContentNode {
        node_type: "paragraph".to_string(),
        attrs: None,
        content: Some(vec![ContentNode {
            node_type: "text".to_string(),
            text: Some(pr_url.to_string()),
            content: None,
            attrs: None,
            marks: Some(vec![Mark {
                mark_type: "link".to_string(),
            }]),
        }]),
        text: None,
        marks: None,
    };

    let mut insert_pos = section_index + 1;
    while insert_pos < checklist.content.len()
        && checklist.content[insert_pos].node_type != "heading"
    {
        insert_pos += 1;
    }

    checklist.content.insert(insert_pos, paragraph);
}

fn remove_pr_url_from_section(checklist: &mut ChecklistField, section_index: usize, pr_url: &str) {
    let mut i = section_index + 1;

    while i < checklist.content.len() {
        let node = &checklist.content[i];

        if node.node_type == "heading" {
            break;
        }

        if node.node_type == "paragraph" {
            if let Some(content) = &node.content {
                for text_node in content {
                    if text_node.node_type == "text" {
                        if let Some(text) = &text_node.text {
                            if text.contains(pr_url) {
                                checklist.content.remove(i);
                                return;
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pr_section() {
        let mut checklist = create_empty_checklist();

        checklist.content.push(ContentNode {
            node_type: "heading".to_string(),
            attrs: Some(NodeAttributes { level: Some(2) }),
            content: Some(vec![ContentNode {
                node_type: "text".to_string(),
                text: Some("Pull Requests".to_string()),
                content: None,
                attrs: None,
                marks: None,
            }]),
            text: None,
            marks: None,
        });

        let index = find_pr_section(&checklist);
        assert_eq!(index, Some(0));
    }

    #[test]
    fn test_pr_url_exists() {
        let mut checklist = create_empty_checklist();

        checklist.content.push(ContentNode {
            node_type: "heading".to_string(),
            attrs: Some(NodeAttributes { level: Some(2) }),
            content: Some(vec![ContentNode {
                node_type: "text".to_string(),
                text: Some("Pull Requests".to_string()),
                content: None,
                attrs: None,
                marks: None,
            }]),
            text: None,
            marks: None,
        });

        checklist.content.push(ContentNode {
            node_type: "paragraph".to_string(),
            attrs: None,
            content: Some(vec![ContentNode {
                node_type: "text".to_string(),
                text: Some("https://github.com/org/repo/pull/1".to_string()),
                content: None,
                attrs: None,
                marks: Some(vec![Mark {
                    mark_type: "link".to_string(),
                }]),
            }]),
            text: None,
            marks: None,
        });

        assert!(pr_url_exists(
            &checklist,
            0,
            "https://github.com/org/repo/pull/1"
        ));
        assert!(!pr_url_exists(
            &checklist,
            0,
            "https://github.com/org/repo/pull/2"
        ));
    }
}
