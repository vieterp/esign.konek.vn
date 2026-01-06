//! PKCS#11 type definitions
//!
//! Defines structs for library detection, token info, and certificates.

use serde::{Deserialize, Serialize};

/// Detected PKCS#11 library information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLibrary {
    pub ca_name: String,
    pub path: String,
}

/// Token information returned from slot enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub slot_id: u64,
    pub label: String,
    pub manufacturer: String,
    pub model: String,
    pub serial: String,
    pub has_token: bool,
}

/// Certificate information from token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub serial: String,
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
    pub thumbprint: String,
    /// DER-encoded certificate bytes (base64)
    pub der_base64: String,
}

/// Format Unix timestamp as Vietnamese datetime format
/// Format: HH:mm:ss dd/MM/yyyy (VNPT-CA standard)
pub fn format_datetime(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    let dt = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(Utc::now);
    dt.format("%H:%M:%S %d/%m/%Y").to_string()
}
