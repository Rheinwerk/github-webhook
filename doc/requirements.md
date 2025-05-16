# github-hook-lambda

This is an AWS Lambda that accepts GitHub Webhooks of type `pull_request`. The goal is to look up the related issue in
Jira and update a custom field, if the pull request is not already linked.


## general guidance

We implement error types using the `thiserror` crate. We don't use comments. We write tests only for functions and structs that are testable in isolation.

## http handler

First, validate `X-Hub-Signature-256` header with secret from environment variable `WEBHOOK_SECRET`. We use the rust
newtype pattern to ensure that the webhook secret is not empty.

We respond 200 to all messages that have no webhook type or are not `pull_request`.

This is the only HTTP/Lambda specific part. Further handling should use appropriate Result types and not return http responses.

## event handler

The other events are further processed. We extract the issue key from the title. The title format is as follows: We
start with one or more comma-separated issue keys in brackets which are followed by a short description.
`[CDTEST-123,CDTEST-234] This is a description`. We extract all issue keys and then query the jira rest v3 api to find
the contents of the field `customfield_10369`. We parse the field into structs according to the Jira Smart Checklist
Plugin's Checklist field format. The field uses the Atlassian Document Format (ADF).

We check if the Checklist custom field `customfield_10369` contains the url to the pull request in a line after the
`# Pull Requests` header. If not, the URL is added. If the event indicates that the title was changed, we also check
against the old title to remove the link to the pull request from all issues that are no longer referenced.
