use crate::error::Error;
use crate::types::WebhookSecret;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn validate_signature(
    payload: &[u8],
    signature_header: Option<&str>,
    secret: &WebhookSecret,
) -> Result<(), Error> {
    let signature = signature_header.ok_or(Error::MissingSignatureHeader)?;

    let signature = signature
        .strip_prefix("sha256=")
        .ok_or(Error::InvalidWebhookSignature)?;

    let signature_bytes = hex::decode(signature).map_err(|_| Error::InvalidWebhookSignature)?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| Error::Internal("Failed to create HMAC".to_string()))?;

    mac.update(payload);

    mac.verify_slice(&signature_bytes)
        .map_err(|_| Error::InvalidWebhookSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_signature_valid() {
        let secret = WebhookSecret::new(b"It's a Secret to Everybody").unwrap();
        let payload = b"Hello, World!";
        let signature = "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

        validate_signature(payload, Some(signature), &secret).expect("Expected Ok");
    }

    #[test]
    fn test_validate_signature_invalid() {
        let secret = WebhookSecret::new("test_secret").unwrap();
        let payload = b"test_payload";

        let signature = "sha256=invalid_signature";

        validate_signature(payload, Some(signature), &secret).expect_err("Expected error");
    }

    #[test]
    fn test_validate_signature_missing_header() {
        let secret = WebhookSecret::new("test_secret").unwrap();
        let payload = b"test_payload";

        validate_signature(payload, None, &secret).expect_err("Expected error");
    }
}
