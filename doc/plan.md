# Implementation Plan for GitHub Hook Lambda

This document outlines the implementation plan for the AWS Lambda function that processes GitHub webhooks for pull requests and updates related Jira issues.

## 1. Project Structure

```
github-hook-lambda/
├── src/
│   ├── main.rs             # Lambda entry point
│   ├── http_handler.rs     # HTTP request handling and webhook validation
│   ├── event_handler.rs    # GitHub event processing
│   ├── jira/
│   │   ├── mod.rs          # Jira module exports
│   │   ├── client.rs       # Jira API client
│   │   ├── models.rs       # Jira data models
│   │   └── checklist.rs    # Checklist field parsing and manipulation
│   ├── github/
│   │   ├── mod.rs          # GitHub module exports
│   │   ├── models.rs       # GitHub webhook payload models
│   │   └── signature.rs    # Signature validation
│   ├── error.rs            # Error types using thiserror
│   └── types.rs            # Common types and newtypes
└── tests/                  # Integration tests
```

## 2. Implementation Steps

### 2.1 Error Handling

- Implement custom error types using the `thiserror` crate
- Create specific error variants for different failure scenarios:
  - Webhook validation errors
  - GitHub event parsing errors
  - Jira API errors
  - Configuration errors

### 2.2 HTTP Handler Implementation

- Create a newtype for `WebhookSecret` that ensures it's not empty
- Implement webhook signature validation using the `X-Hub-Signature-256` header
- Extract the webhook type from the request
- Return 200 for messages with no webhook type or non-pull_request events
- Forward valid pull_request events to the event handler

### 2.3 Event Handler Implementation

- Parse the pull request payload
- Extract issue keys from the PR title using the specified format: `[ISSUE-123,ISSUE-234] Description`
- Handle different pull request actions (opened, edited, etc.)
- For title changes, track both old and new issue references

### 2.4 Jira Integration

- Implement Jira REST API v3 client
- Create models for the Atlassian Document Format (ADF)
- Implement parsing and manipulation of the Smart Checklist Plugin's format
- Query the `customfield_10369` field for each issue
- Check if the PR URL already exists under the "Pull Requests" header
- Add the PR URL if it doesn't exist
- For title changes, remove PR links from issues no longer referenced

### 2.5 Testing

- Write unit tests for functions that can be tested in isolation:
  - Webhook signature validation
  - Issue key extraction from PR titles
  - Checklist field parsing and manipulation
- Create mock implementations for external dependencies

## 3. Implementation Details

### 3.1 Webhook Secret Handling

```rust
pub struct WebhookSecret(String);

impl WebhookSecret {
    pub fn new(secret: String) -> Result<Self, Error> {
        if secret.is_empty() {
            return Err(Error::EmptyWebhookSecret);
        }
        Ok(Self(secret))
    }
}
```

### 3.2 Issue Key Extraction

- Use regex to extract issue keys from PR titles
- Handle multiple comma-separated keys in brackets
- Example: `[CDTEST-123,CDTEST-234] This is a description`

### 3.3 Jira Checklist Field Handling

- Parse the ADF format of the `customfield_10369` field
- Locate the "Pull Requests" section
- Check if the PR URL already exists in this section
- Add the PR URL if it doesn't exist
- For title changes, remove PR URLs from issues no longer referenced

## 4. Deployment Considerations

- Configure appropriate IAM permissions for the Lambda function
- Set up environment variables:
  - `WEBHOOK_SECRET`: Secret for GitHub webhook validation
  - `JIRA_API_TOKEN`: Token for Jira API authentication
  - `JIRA_BASE_URL`: Base URL for the Jira instance
  - `JIRA_USER_EMAIL`: Email for Jira API authentication

## 5. Timeline

1. Set up project structure and error handling (1 day)
2. Implement HTTP handler with webhook validation (1 day)
3. Implement event handler with issue key extraction (1 day)
4. Implement Jira client and ADF parsing (2 days)
5. Implement checklist field manipulation (1 day)
6. Write tests and fix bugs (2 days)
7. Documentation and final review (1 day)

Total estimated time: 9 days