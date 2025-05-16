use crate::error::Error;
use crate::jira::models::{ChecklistField, ContentNode, JiraClient, Mark, NodeAttributes};
use tracing::{debug, info};

/// Checklist manipulator for Jira issues
pub struct ChecklistManipulator {
    client: JiraClient,
}

impl ChecklistManipulator {
    /// Creates a new checklist manipulator with the given Jira client
    pub fn new(client: JiraClient) -> Self {
        Self { client }
    }

    /// Creates a new checklist manipulator from environment variables
    pub fn from_env() -> Result<Self, Error> {
        let client = JiraClient::from_env()?;
        Ok(Self::new(client))
    }

    /// Adds a PR URL to the issue's checklist field
    pub async fn add_pr_url(&self, issue_key: &str, pr_url: &str) -> Result<(), Error> {
        info!("Adding PR URL to issue {}: {}", issue_key, pr_url);
        
        // Get the issue
        let issue = self.client.get_issue(issue_key).await?;
        
        // Get the checklist field or create a new one
        let mut checklist = match issue.fields.checklist {
            Some(checklist) => checklist,
            None => create_empty_checklist(),
        };
        
        // Find or create the "Pull Requests" section
        let pr_section_index = find_or_create_pr_section(&mut checklist);
        
        // Check if the PR URL already exists
        if pr_url_exists(&checklist, pr_section_index, pr_url) {
            debug!("PR URL already exists in issue {}", issue_key);
            return Ok(());
        }
        
        // Add the PR URL
        add_pr_url_to_section(&mut checklist, pr_section_index, pr_url);
        
        // Update the issue
        self.client.update_checklist(issue_key, &checklist).await
    }

    /// Removes a PR URL from the issue's checklist field
    pub async fn remove_pr_url(&self, issue_key: &str, pr_url: &str) -> Result<(), Error> {
        info!("Removing PR URL from issue {}: {}", issue_key, pr_url);
        
        // Get the issue
        let issue = self.client.get_issue(issue_key).await?;
        
        // Get the checklist field
        let Some(mut checklist) = issue.fields.checklist else {
            debug!("Issue {} has no checklist field", issue_key);
            return Ok(());
        };
        
        // Find the "Pull Requests" section
        let Some(pr_section_index) = find_pr_section(&checklist) else {
            debug!("Issue {} has no Pull Requests section", issue_key);
            return Ok(());
        };
        
        // Check if the PR URL exists
        if !pr_url_exists(&checklist, pr_section_index, pr_url) {
            debug!("PR URL does not exist in issue {}", issue_key);
            return Ok(());
        }
        
        // Remove the PR URL
        remove_pr_url_from_section(&mut checklist, pr_section_index, pr_url);
        
        // Update the issue
        self.client.update_checklist(issue_key, &checklist).await
    }
}

/// Creates an empty checklist field
fn create_empty_checklist() -> ChecklistField {
    ChecklistField {
        doc_type: "doc".to_string(),
        version: 1,
        content: Vec::new(),
    }
}

/// Finds the "Pull Requests" section in the checklist
fn find_pr_section(checklist: &ChecklistField) -> Option<usize> {
    checklist.content.iter().position(|node| {
        node.node_type == "heading" 
            && node.content.as_ref().map_or(false, |content| {
                content.iter().any(|text_node| {
                    text_node.node_type == "text" 
                        && text_node.text.as_ref().map_or(false, |text| {
                            text.trim() == "Pull Requests"
                        })
                })
            })
    })
}

/// Finds or creates the "Pull Requests" section in the checklist
fn find_or_create_pr_section(checklist: &mut ChecklistField) -> usize {
    if let Some(index) = find_pr_section(checklist) {
        return index;
    }
    
    // Create a new "Pull Requests" heading
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
    
    // Add the heading to the checklist
    checklist.content.push(heading);
    
    // Return the index of the new heading
    checklist.content.len() - 1
}

/// Checks if a PR URL exists in the section after the given index
fn pr_url_exists(checklist: &ChecklistField, section_index: usize, pr_url: &str) -> bool {
    // Look for paragraph nodes after the section heading
    for i in (section_index + 1)..checklist.content.len() {
        let node = &checklist.content[i];
        
        // Stop if we hit another heading
        if node.node_type == "heading" {
            break;
        }
        
        // Check if this paragraph contains the PR URL
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

/// Adds a PR URL to the section after the given index
fn add_pr_url_to_section(checklist: &mut ChecklistField, section_index: usize, pr_url: &str) {
    // Create a new paragraph with the PR URL
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
    
    // Find the position to insert the paragraph
    let mut insert_pos = section_index + 1;
    while insert_pos < checklist.content.len() && checklist.content[insert_pos].node_type != "heading" {
        insert_pos += 1;
    }
    
    // Insert the paragraph
    checklist.content.insert(insert_pos, paragraph);
}

/// Removes a PR URL from the section after the given index
fn remove_pr_url_from_section(checklist: &mut ChecklistField, section_index: usize, pr_url: &str) {
    let mut i = section_index + 1;
    
    while i < checklist.content.len() {
        let node = &checklist.content[i];
        
        // Stop if we hit another heading
        if node.node_type == "heading" {
            break;
        }
        
        // Check if this paragraph contains the PR URL
        if node.node_type == "paragraph" {
            if let Some(content) = &node.content {
                for text_node in content {
                    if text_node.node_type == "text" {
                        if let Some(text) = &text_node.text {
                            if text.contains(pr_url) {
                                // Remove this paragraph
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
        
        // Add a heading
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
        
        // Find the section
        let index = find_pr_section(&checklist);
        assert_eq!(index, Some(0));
    }
    
    #[test]
    fn test_pr_url_exists() {
        let mut checklist = create_empty_checklist();
        
        // Add a heading
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
        
        // Add a paragraph with a PR URL
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
        
        // Check if the PR URL exists
        assert!(pr_url_exists(&checklist, 0, "https://github.com/org/repo/pull/1"));
        assert!(!pr_url_exists(&checklist, 0, "https://github.com/org/repo/pull/2"));
    }
}