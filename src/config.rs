use crate::error::Error;
use crate::jira::models::JiraConfig;
use crate::types::WebhookSecret;
use aws_sdk_kms::primitives::Blob;

pub struct Config {
    pub dry_run: bool,
    pub webhook_secret: WebhookSecret,
    pub jira_config: JiraConfig,
}

const DRY_RUN: &str = "DRY_RUN";
const WEBHOOK_SECRET: &str = "WEBHOOK_SECRET";
const WEBHOOK_SECRET_KMS: &str = "WEBHOOK_SECRET_KMS";
const JIRA_URL: &str = "JIRA_URL";
const JIRA_EMAIL: &str = "JIRA_EMAIL";
const JIRA_TOKEN: &str = "JIRA_TOKEN";
const JIRA_TOKEN_KMS: &str = "JIRA_TOKEN_KMS";

impl Config {
    pub async fn from_env(aws_kms: &aws_sdk_kms::Client) -> Result<Self, Error> {
        use std::env::{var, VarError};

        let dry_run = match var(DRY_RUN) {
            Err(VarError::NotPresent) => false,
            Ok(value) => !value.is_empty(),
            Err(VarError::NotUnicode(var)) => !var.is_empty(),
        };

        let webhook_secret = get_encrypted_var(WEBHOOK_SECRET, WEBHOOK_SECRET_KMS, &aws_kms)
            .await
            .and_then(WebhookSecret::new)?;

        let jira_email = var(JIRA_EMAIL).map_err(|_| Error::EnvVarNotSet {
            env_var_name: JIRA_EMAIL,
        })?;
        let jira_token = get_encrypted_var(JIRA_TOKEN, JIRA_TOKEN_KMS, &aws_kms).await?;
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

async fn get_encrypted_var(
    plain_text_name: &'static str,
    encrypted_name: &'static str,
    aws_kms: &aws_sdk_kms::Client,
) -> Result<String, Error> {
    if let Ok(encrypted_value) = std::env::var(encrypted_name) {
        use base64::{engine::general_purpose::STANDARD as Base64, Engine as _};
        let encrypted_bytes =
            Base64
                .decode(encrypted_value)
                .map_err(|_| Error::EnvVarBadValue {
                    env_var_name: encrypted_name,
                })?;

        let decrypted = aws_kms
            .decrypt()
            .ciphertext_blob(Blob::from(encrypted_bytes))
            .send()
            .await
            .map_err(|e| Error::AwsKms(e.into()))?;

        let decrypted = decrypted.plaintext().ok_or(Error::Internal(
            "decrypted value had no plain text".to_string(),
        ))?;

        return Ok(String::from_utf8_lossy(decrypted.as_ref()).to_string());
    };

    // fall back to plain text
    std::env::var(plain_text_name).map_err(|_| Error::EnvVarNotSet {
        env_var_name: plain_text_name,
    })
}
