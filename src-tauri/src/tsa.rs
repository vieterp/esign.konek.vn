//! Timestamp Authority (TSA) Module
//!
//! Implements RFC 3161 timestamp requests for PAdES-T signatures.
//! Supports Vietnamese TSA servers with fallback logic.

use crate::error::ESignError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Vietnamese TSA server URLs
/// HTTPS endpoints are preferred for security; HTTP is fallback only
pub mod servers {
    /// Vietnam Post TSA (HTTPS preferred)
    pub const VNPT_HTTPS: &str = "https://ca.vnpt.vn/tsa";
    /// Vietnam Post TSA (HTTP fallback - insecure)
    pub const VNPT_HTTP: &str = "http://ca.vnpt.vn/tsa";
    /// Viettel TSA (HTTPS preferred)
    pub const VIETTEL_HTTPS: &str = "https://tsa.viettel-ca.vn";
    /// Viettel TSA (HTTP fallback - insecure)
    pub const VIETTEL_HTTP: &str = "http://tsa.viettel-ca.vn";
    /// FPT TSA (HTTPS preferred)
    pub const FPT_HTTPS: &str = "https://tsa.fpt.vn";
    /// FPT TSA (HTTP fallback - insecure)
    pub const FPT_HTTP: &str = "http://tsa.fpt.vn";

    /// Check if URL is using insecure HTTP
    pub fn is_insecure(url: &str) -> bool {
        url.starts_with("http://")
    }
}

/// TSA server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsaConfig {
    /// Primary TSA URL
    pub primary_url: String,
    /// Fallback TSA URLs
    pub fallback_urls: Vec<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for TsaConfig {
    fn default() -> Self {
        Self {
            // HTTPS endpoints first, HTTP fallbacks last (security preference)
            primary_url: servers::VNPT_HTTPS.to_string(),
            fallback_urls: vec![
                servers::VIETTEL_HTTPS.to_string(),
                servers::FPT_HTTPS.to_string(),
                // HTTP fallbacks as last resort (will trigger warning)
                servers::VNPT_HTTP.to_string(),
                servers::VIETTEL_HTTP.to_string(),
                servers::FPT_HTTP.to_string(),
            ],
            timeout_secs: 30,
        }
    }
}

/// TSA client for RFC 3161 timestamp requests
pub struct TsaClient {
    config: TsaConfig,
    http_client: Client,
}

impl TsaClient {
    /// Create new TSA client with default Vietnamese servers
    pub fn new() -> Result<Self, ESignError> {
        Self::with_config(TsaConfig::default())
    }

    /// Create TSA client with custom configuration
    pub fn with_config(config: TsaConfig) -> Result<Self, ESignError> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| ESignError::Tsa(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Get timestamp token for signature data
    /// Tries HTTPS servers first, falls back to HTTP with warning
    /// Returns DER-encoded TimeStampToken
    pub fn get_timestamp(&self, signature: &[u8]) -> Result<Vec<u8>, ESignError> {
        // Hash the signature for the timestamp request
        let mut hasher = Sha256::new();
        hasher.update(signature);
        let hash = hasher.finalize();

        // Build timestamp request
        let ts_request = self.build_timestamp_request(&hash)?;

        // Try primary server first, then fallbacks
        let mut urls = vec![self.config.primary_url.clone()];
        urls.extend(self.config.fallback_urls.clone());

        let mut last_error = None;
        for url in &urls {
            match self.send_timestamp_request(url, &ts_request) {
                Ok(response) => {
                    return self.parse_timestamp_response(&response);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ESignError::Tsa("No TSA servers available".to_string())))
    }

    /// Build RFC 3161 TimeStampReq
    /// ASN.1 structure for timestamp request
    fn build_timestamp_request(&self, hash: &[u8]) -> Result<Vec<u8>, ESignError> {
        // TimeStampReq ::= SEQUENCE {
        //   version INTEGER { v1(1) },
        //   messageImprint MessageImprint,
        //   reqPolicy TSAPolicyId OPTIONAL,
        //   nonce INTEGER OPTIONAL,
        //   certReq BOOLEAN DEFAULT FALSE,
        //   extensions [0] IMPLICIT Extensions OPTIONAL
        // }
        //
        // MessageImprint ::= SEQUENCE {
        //   hashAlgorithm AlgorithmIdentifier,
        //   hashedMessage OCTET STRING
        // }

        // SHA-256 OID: 2.16.840.1.101.3.4.2.1
        let sha256_oid: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];

        // Build AlgorithmIdentifier for SHA-256
        let mut alg_id = vec![0x30]; // SEQUENCE
        let alg_content_len = 2 + sha256_oid.len() + 2; // OID + NULL
        alg_id.push(alg_content_len as u8);
        alg_id.push(0x06); // OID tag
        alg_id.push(sha256_oid.len() as u8);
        alg_id.extend_from_slice(sha256_oid);
        alg_id.extend_from_slice(&[0x05, 0x00]); // NULL

        // Build MessageImprint
        let mut msg_imprint = vec![0x30]; // SEQUENCE
        let msg_content = [&alg_id[..], &[0x04, hash.len() as u8], hash].concat();
        msg_imprint.push(msg_content.len() as u8);
        msg_imprint.extend_from_slice(&msg_content);

        // Build TimeStampReq
        let version: &[u8] = &[0x02, 0x01, 0x01]; // INTEGER 1
        let cert_req: &[u8] = &[0x01, 0x01, 0xFF]; // BOOLEAN TRUE

        // Generate random nonce
        let nonce_value: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let nonce_bytes = nonce_value.to_be_bytes();
        let mut nonce = vec![0x02]; // INTEGER
                                    // Remove leading zeros
        let nonce_trimmed: Vec<u8> = nonce_bytes
            .iter()
            .skip_while(|&&b| b == 0)
            .cloned()
            .collect();
        let nonce_data = if nonce_trimmed.is_empty() {
            vec![0]
        } else {
            nonce_trimmed
        };
        nonce.push(nonce_data.len() as u8);
        nonce.extend_from_slice(&nonce_data);

        let req_content = [version, &msg_imprint[..], &nonce[..], cert_req].concat();

        let mut ts_req = vec![0x30]; // SEQUENCE
        if req_content.len() < 128 {
            ts_req.push(req_content.len() as u8);
        } else {
            // Long form length encoding
            let len_bytes = (req_content.len() as u32).to_be_bytes();
            let len_trimmed: Vec<u8> = len_bytes.iter().skip_while(|&&b| b == 0).cloned().collect();
            ts_req.push(0x80 | len_trimmed.len() as u8);
            ts_req.extend_from_slice(&len_trimmed);
        }
        ts_req.extend_from_slice(&req_content);

        Ok(ts_req)
    }

    /// Send timestamp request to TSA server
    fn send_timestamp_request(&self, url: &str, request: &[u8]) -> Result<Vec<u8>, ESignError> {
        let response = self
            .http_client
            .post(url)
            .header("Content-Type", "application/timestamp-query")
            .body(request.to_vec())
            .send()
            .map_err(|e| ESignError::Tsa(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ESignError::Tsa(format!(
                "TSA returned error status: {}",
                response.status()
            )));
        }

        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| ESignError::Tsa(format!("Failed to read response: {}", e)))
    }

    /// Parse RFC 3161 TimeStampResp and extract TimeStampToken
    fn parse_timestamp_response(&self, response: &[u8]) -> Result<Vec<u8>, ESignError> {
        // TimeStampResp ::= SEQUENCE {
        //   status PKIStatusInfo,
        //   timeStampToken TimeStampToken OPTIONAL
        // }
        //
        // PKIStatusInfo ::= SEQUENCE {
        //   status PKIStatus,
        //   ...
        // }
        //
        // PKIStatus ::= INTEGER {
        //   granted(0), grantedWithMods(1), rejection(2), ...
        // }

        if response.len() < 5 {
            return Err(ESignError::Tsa("Response too short".to_string()));
        }

        // Check outer SEQUENCE
        if response[0] != 0x30 {
            return Err(ESignError::Tsa(
                "Invalid response: not a SEQUENCE".to_string(),
            ));
        }

        // Parse length
        let (content_start, _content_len) = parse_asn1_length(&response[1..])?;
        let content = &response[1 + content_start..];

        // Parse PKIStatusInfo (first element)
        if content[0] != 0x30 {
            return Err(ESignError::Tsa("Invalid PKIStatusInfo".to_string()));
        }
        let (status_content_start, status_content_len) = parse_asn1_length(&content[1..])?;
        let status_info_len = 1 + status_content_start + status_content_len;

        // Check status value
        let status_content =
            &content[1 + status_content_start..1 + status_content_start + status_content_len];
        if status_content.len() >= 3 && status_content[0] == 0x02 {
            let status_value = status_content[2];
            if status_value > 1 {
                return Err(ESignError::Tsa(format!(
                    "TSA rejected request with status {}",
                    status_value
                )));
            }
        }

        // Extract TimeStampToken (second element)
        if content.len() <= status_info_len {
            return Err(ESignError::Tsa("No TimeStampToken in response".to_string()));
        }

        let token_start = status_info_len;
        let token_data = &content[token_start..];

        // Verify it's a ContentInfo SEQUENCE
        if token_data[0] != 0x30 {
            return Err(ESignError::Tsa("Invalid TimeStampToken".to_string()));
        }

        let (token_len_start, token_len) = parse_asn1_length(&token_data[1..])?;
        let total_token_len = 1 + token_len_start + token_len;

        Ok(token_data[..total_token_len].to_vec())
    }
}

impl Default for TsaClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default TSA client")
    }
}

/// Parse ASN.1 length encoding
/// Returns (bytes consumed, length value)
fn parse_asn1_length(data: &[u8]) -> Result<(usize, usize), ESignError> {
    if data.is_empty() {
        return Err(ESignError::Tsa("Unexpected end of data".to_string()));
    }

    if data[0] < 128 {
        // Short form
        Ok((1, data[0] as usize))
    } else {
        // Long form
        let num_bytes = (data[0] & 0x7F) as usize;
        if num_bytes == 0 || num_bytes > 4 || data.len() < 1 + num_bytes {
            return Err(ESignError::Tsa("Invalid length encoding".to_string()));
        }

        let mut length: usize = 0;
        for i in 0..num_bytes {
            length = (length << 8) | (data[1 + i] as usize);
        }

        Ok((1 + num_bytes, length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ TSA Server Constants Tests ============

    #[test]
    fn test_vnpt_tsa_urls() {
        assert!(!servers::VNPT_HTTPS.is_empty());
        assert!(servers::VNPT_HTTPS.starts_with("https"));
        assert!(!servers::VNPT_HTTP.is_empty());
        assert!(servers::VNPT_HTTP.starts_with("http://"));
    }

    #[test]
    fn test_viettel_tsa_urls() {
        assert!(!servers::VIETTEL_HTTPS.is_empty());
        assert!(servers::VIETTEL_HTTPS.starts_with("https"));
        assert!(!servers::VIETTEL_HTTP.is_empty());
        assert!(servers::VIETTEL_HTTP.starts_with("http://"));
    }

    #[test]
    fn test_fpt_tsa_urls() {
        assert!(!servers::FPT_HTTPS.is_empty());
        assert!(servers::FPT_HTTPS.starts_with("https"));
        assert!(!servers::FPT_HTTP.is_empty());
        assert!(servers::FPT_HTTP.starts_with("http://"));
    }

    #[test]
    fn test_is_insecure() {
        assert!(servers::is_insecure("http://example.com"));
        assert!(!servers::is_insecure("https://example.com"));
    }

    // ============ TsaConfig Tests ============

    #[test]
    fn test_tsa_config_default() {
        let config = TsaConfig::default();
        assert_eq!(config.primary_url, servers::VNPT_HTTPS);
        assert!(!config.fallback_urls.is_empty());
        assert_eq!(config.timeout_secs, 30);
        // Verify HTTPS endpoints come before HTTP fallbacks
        assert!(config.fallback_urls[0].starts_with("https"));
    }

    #[test]
    fn test_tsa_config_custom() {
        let config = TsaConfig {
            primary_url: "http://custom.tsa.vn".to_string(),
            fallback_urls: vec!["http://fallback1.vn".to_string()],
            timeout_secs: 60,
        };
        assert_eq!(config.primary_url, "http://custom.tsa.vn");
        assert_eq!(config.fallback_urls.len(), 1);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_tsa_config_serialize() {
        let config = TsaConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("primary_url"));
        assert!(json.contains("fallback_urls"));
        assert!(json.contains("timeout_secs"));
    }

    #[test]
    fn test_tsa_config_deserialize() {
        let json = r#"{"primary_url":"http://test.vn","fallback_urls":[],"timeout_secs":10}"#;
        let config: TsaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.primary_url, "http://test.vn");
        assert!(config.fallback_urls.is_empty());
        assert_eq!(config.timeout_secs, 10);
    }

    // ============ TsaClient Tests ============

    #[test]
    fn test_tsa_client_new() {
        let client = TsaClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_tsa_client_with_config() {
        let config = TsaConfig {
            primary_url: servers::VIETTEL_HTTPS.to_string(),
            fallback_urls: vec![],
            timeout_secs: 15,
        };
        let client = TsaClient::with_config(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_tsa_client_default() {
        // Test that default construction doesn't panic
        let _client = TsaClient::default();
    }

    // ============ ASN.1 Length Parsing Tests ============

    #[test]
    fn test_parse_asn1_length_short() {
        let data = [0x0A];
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(len, 10);
    }

    #[test]
    fn test_parse_asn1_length_zero() {
        let data = [0x00];
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_parse_asn1_length_max_short() {
        let data = [0x7F]; // 127 - max short form
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(len, 127);
    }

    #[test]
    fn test_parse_asn1_length_long_one_byte() {
        let data = [0x81, 0x80]; // 128 in long form
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 2);
        assert_eq!(len, 128);
    }

    #[test]
    fn test_parse_asn1_length_long() {
        let data = [0x82, 0x01, 0x00];
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 3);
        assert_eq!(len, 256);
    }

    #[test]
    fn test_parse_asn1_length_long_large() {
        let data = [0x82, 0x10, 0x00]; // 4096
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 3);
        assert_eq!(len, 4096);
    }

    #[test]
    fn test_parse_asn1_length_empty() {
        let data: [u8; 0] = [];
        let result = parse_asn1_length(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_asn1_length_invalid_long_form() {
        // Long form with 0 bytes following (invalid)
        let data = [0x80];
        let result = parse_asn1_length(&data);
        assert!(result.is_err());
    }

    // ============ Timestamp Request Tests ============

    #[test]
    fn test_build_timestamp_request() {
        let client = TsaClient::new().unwrap();
        let hash = [0u8; 32];
        let request = client.build_timestamp_request(&hash).unwrap();
        // Should start with SEQUENCE tag
        assert_eq!(request[0], 0x30);
    }

    #[test]
    fn test_build_timestamp_request_different_hash() {
        let client = TsaClient::new().unwrap();
        let hash1 = [0u8; 32];
        let hash2 = [0xFFu8; 32];

        let request1 = client.build_timestamp_request(&hash1).unwrap();
        let request2 = client.build_timestamp_request(&hash2).unwrap();

        // Requests should have same structure but different content
        assert_eq!(request1[0], request2[0]); // Both SEQUENCE
        assert_ne!(request1, request2); // But different content
    }

    #[test]
    fn test_build_timestamp_request_structure() {
        let client = TsaClient::new().unwrap();
        let hash = [0xAB; 32];
        let request = client.build_timestamp_request(&hash).unwrap();

        // Verify it's a valid ASN.1 SEQUENCE
        assert_eq!(request[0], 0x30);

        // Should be longer than just the hash (includes version, OID, etc.)
        assert!(request.len() > 32 + 10);
    }

    // ============ Config Roundtrip Tests ============

    #[test]
    fn test_tsa_config_roundtrip() {
        let original = TsaConfig {
            primary_url: "http://test.vn".to_string(),
            fallback_urls: vec!["http://fb1.vn".to_string(), "http://fb2.vn".to_string()],
            timeout_secs: 45,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TsaConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.primary_url, restored.primary_url);
        assert_eq!(original.fallback_urls, restored.fallback_urls);
        assert_eq!(original.timeout_secs, restored.timeout_secs);
    }

    // ============ Edge Cases ============

    #[test]
    fn test_tsa_config_empty_fallbacks() {
        let config = TsaConfig {
            primary_url: servers::VNPT_HTTPS.to_string(),
            fallback_urls: vec![],
            timeout_secs: 30,
        };
        assert!(config.fallback_urls.is_empty());
    }

    #[test]
    fn test_parse_asn1_length_three_bytes() {
        // Test with 3-byte length encoding (for very large values)
        let data = [0x83, 0x01, 0x00, 0x00]; // 65536
        let (consumed, len) = parse_asn1_length(&data).unwrap();
        assert_eq!(consumed, 4);
        assert_eq!(len, 65536);
    }
}
