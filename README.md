# GitHub Hook Lambda

GitHub Hook Lambda is an AWS Lambda function that processes GitHub webhooks for pull requests and updates related Jira
issues. When a pull request is created or its title is updated, the Lambda function extracts issue keys from the PR
title and adds a link to the PR in the corresponding Jira issues' checklist field.

## Features

- Validates GitHub webhook signatures for security
- Extracts issue keys from PR titles (format: `[ISSUE-123,ISSUE-234] Description`)
- Updates Jira issues with links to pull requests

## Configuration

The Lambda function requires the following environment variables:


- `RUST_LOG`: Controls logging

- `DRY_RUN`: Disables issue mutation (must be set to a nonemtpy value)

- `JIRA_BASE_URL`: Base URL for the Jira instance (e.g., `https://your-company.atlassian.net`)
- `JIRA_USER_EMAIL`: Email for Jira API authentication
- `JIRA_API_TOKEN`: Token for Jira API authentication
- `JIRA_API_TOKEN`: Token for Jira API authentication, AWS KMS encrypted

- `WEBHOOK_SECRET`: Secret for GitHub webhook validation
- `WEBHOOK_SECRET_KMS`: Secret for GitHub webhook validation, AWS KMS encrypted

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

## Building

To build the project for production, run `cargo lambda build --release --output-format zip --arm64`.

Read more about building your lambda function
in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/build.html).

## Testing

You can run regular Rust unit tests with `cargo test`.

If you want to run integration tests locally, you can use the `cargo lambda watch` and `cargo lambda invoke` commands to
do it.

First, run `cargo lambda watch` to start a local server. When you make changes to the code, the server will
automatically restart.

Second, you'll need a way to pass the event data to the lambda function.

You can use the
existent [event payloads](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events/src/fixtures) in
the Rust Runtime repository if your lambda function is using one of the supported event types.

You can use those examples directly with the `--data-example` flag, where the value is the name of the file in
the [lambda-events](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-events/src/fixtures) repository
without the `example_` prefix and the `.json` extension.

```bash
cargo lambda invoke --data-example apigw-request
```

For generic events, where you define the event data structure, you can create a JSON file with the data you want to test
with. For example:

```json
{
  "command": "test"
}
```

Then, run `cargo lambda invoke --data-file ./data.json` to invoke the function with the data in `data.json`.

For HTTP events, you can also call the function directly with cURL or any other HTTP client. For example:

```bash
curl https://localhost:9000
```

Read more about running the local server in [the Cargo Lambda documentation for the
`watch` command](https://www.cargo-lambda.info/commands/watch.html). Read more about invoking the function
in [the Cargo Lambda documentation for the `invoke` command](https://www.cargo-lambda.info/commands/invoke.html).
