use crate::error::Error;
use crate::jira::models::JiraConfig;
use crate::types::WebhookSecret;

pub struct Config {
    pub dry_run: bool,
    pub webhook_secret: WebhookSecret,
    pub jira_config: JiraConfig,
}

const DRY_RUN: &str = "DRY_RUN";
const WEBHOOK_SECRET: &str = "WEBHOOK_SECRET";
const JIRA_URL: &str = "JIRA_URL";
const JIRA_EMAIL: &str = "JIRA_EMAIL";
const JIRA_TOKEN: &str = "JIRA_TOKEN";

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        use std::env::{var, VarError};

        let dry_run = match var(DRY_RUN) {
            Err(VarError::NotPresent) => false,
            Ok(value) => !value.is_empty(),
            Err(VarError::NotUnicode(var)) => !var.is_empty(),
        };

        let webhook_secret = var(WEBHOOK_SECRET)
            .map_err(|_| Error::EnvVarNotSet {
                env_var_name: WEBHOOK_SECRET,
            })
            .and_then(WebhookSecret::new)?;

        let jira_email = var(JIRA_EMAIL).map_err(|_| Error::EnvVarNotSet {
            env_var_name: JIRA_EMAIL,
        })?;
        let jira_token = var(JIRA_TOKEN).map_err(|_| Error::EnvVarNotSet {
            env_var_name: JIRA_TOKEN,
        })?;
        let jira_url = var(JIRA_URL)
            .map_err(|_| Error::EnvVarNotSet {
                env_var_name: JIRA_URL,
            })
            .and_then(|url| {
                reqwest::Url::parse(&url).map_err(|_| Error::EnvVarBadValue {
                    env_var_name: JIRA_URL,
                })
            })
            .map(|mut url| {
                // Url must end in a slash so that it can be used as a base for other urls
                if !url.path().ends_with("/") {
                    url.set_path(&format!("{}/", url.path()));
                }
                url
            })?;

        Ok(Config {
            jira_config: JiraConfig {
                email: jira_email,
                api_token: jira_token,
                base_url: jira_url,
            },
            webhook_secret,
            dry_run,
        })
    }
}
