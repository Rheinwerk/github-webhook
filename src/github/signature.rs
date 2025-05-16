use crate::error::Error;
use crate::types::WebhookSecret;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::str::FromStr;

type HmacSha256 = Hmac<Sha256>;

/// Validates the GitHub webhook signature
pub fn validate_signature(
    payload: &[u8],
    signature_header: Option<&str>,
    secret: &WebhookSecret,
) -> Result<(), Error> {
    let signature = signature_header.ok_or(Error::MissingSignatureHeader)?;
    
    // GitHub signature header format: "sha256=<hex_signature>"
    let signature = signature
        .strip_prefix("sha256=")
        .ok_or(Error::InvalidWebhookSignature)?;
    
    // Decode the hex signature
    let signature_bytes = hex::decode(signature).map_err(|_| Error::InvalidWebhookSignature)?;
    
    // Create HMAC with the secret
    let mut mac = HmacSha256::new_from_slice(secret.as_str().as_bytes())
        .map_err(|_| Error::InternalError("Failed to create HMAC".to_string()))?;
    
    // Update with payload
    mac.update(payload);
    
    // Verify the signature
    mac.verify_slice(&signature_bytes)
        .map_err(|_| Error::InvalidWebhookSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_signature_valid() {
        let secret = WebhookSecret::new("test_secret".to_string()).unwrap();
        let payload = b"test_payload";
        
        // Generate a valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_str().as_bytes()).unwrap();
        mac.update(payload);
        let signature_bytes = mac.finalize().into_bytes();
        let signature = format!("sha256={}", hex::encode(signature_bytes));
        
        // Validate the signature
        let result = validate_signature(payload, Some(&signature), &secret);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_signature_invalid() {
        let secret = WebhookSecret::new("test_secret".to_string()).unwrap();
        let payload = b"test_payload";
        
        // Invalid signature
        let signature = "sha256=invalid_signature";
        
        // Validate the signature
        let result = validate_signature(payload, Some(signature), &secret);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidWebhookSignature));
    }
    
    #[test]
    fn test_validate_signature_missing_header() {
        let secret = WebhookSecret::new("test_secret".to_string()).unwrap();
        let payload = b"test_payload";
        
        // Missing signature header
        let result = validate_signature(payload, None, &secret);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::MissingSignatureHeader));
    }
}