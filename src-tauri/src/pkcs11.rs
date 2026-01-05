//! PKCS#11 USB Token Communication Module
//!
//! Handles communication with Vietnamese CA USB tokens (VNPT, Viettel, FPT)
//! using the PKCS#11 standard via the cryptoki crate.

use crate::error::{ESignError, SigningErrorCode};
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType, ObjectClass, ObjectHandle},
    session::{Session, UserType},
    slot::Slot,
    types::AuthPin,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use x509_parser::prelude::*;
use zeroize::Zeroize;

/// Known PKCS#11 library paths for Vietnamese CAs
pub mod library_paths {
    /// All known library paths for auto-detection
    pub fn all_paths() -> Vec<(&'static str, &'static str)> {
        vec![
            ("VNPT-CA", vnpt::PATH),
            ("Viettel-CA", viettel::PATH),
            ("FPT-CA", fpt::PATH),
        ]
    }

    /// VNPT-CA PKCS#11 library paths
    pub mod vnpt {
        #[cfg(target_os = "macos")]
        pub const PATH: &str = "/Library/vnpt-ca/lib/libcryptoki.dylib";
        #[cfg(target_os = "windows")]
        pub const PATH: &str = "C:\\vnpt-ca\\cryptoki.dll";
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        pub const PATH: &str = "/usr/lib/vnpt-ca/libcryptoki.so";
    }

    /// Viettel-CA PKCS#11 library paths
    pub mod viettel {
        #[cfg(target_os = "macos")]
        pub const PATH: &str = "/Library/viettel-ca/libpkcs11.dylib";
        #[cfg(target_os = "windows")]
        pub const PATH: &str = "C:\\Viettel-CA\\pkcs11.dll";
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        pub const PATH: &str = "/usr/lib/viettel-ca/libpkcs11.so";
    }

    /// FPT-CA PKCS#11 library paths
    pub mod fpt {
        #[cfg(target_os = "macos")]
        pub const PATH: &str = "/Library/FPT/libpkcs11.dylib";
        #[cfg(target_os = "windows")]
        pub const PATH: &str = "C:\\FPT-CA\\pkcs11.dll";
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        pub const PATH: &str = "/usr/lib/fpt-ca/libpkcs11.so";
    }
}

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

/// Token manager - handles PKCS#11 operations
/// Thread-safe wrapper around cryptoki session
pub struct TokenManager {
    ctx: Pkcs11,
    session: Mutex<Option<Session>>,
    signing_key: Mutex<Option<ObjectHandle>>,
    certificate_der: Mutex<Option<Vec<u8>>>,
    library_path: String,
}

impl TokenManager {
    /// Create architecture mismatch error with helpful guidance
    /// Parses the error message to extract architecture info and provides actionable advice
    fn create_arch_mismatch_error(error_str: &str, library_path: &str) -> ESignError {
        // Parse architectures from error: "have 'x86_64', need 'arm64e' or 'arm64'"
        let (library_arch, host_arch) = Self::parse_arch_from_error(error_str);

        // Generate platform-specific guidance
        let guidance = if host_arch.contains("arm64") && library_arch.contains("x86_64") {
            // Apple Silicon Mac with x86_64 library
            format!(
                "Thư viện PKCS#11 của nhà cung cấp chỉ hỗ trợ Intel (x86_64). \
                Giải pháp: (1) Liên hệ nhà cung cấp CA để xin phiên bản ARM64, \
                hoặc (2) Chạy ứng dụng qua Rosetta 2: arch -x86_64 open -a \"Konek eSign\""
            )
        } else if host_arch.contains("x86_64") && library_arch.contains("arm64") {
            // Intel Mac with ARM64 library
            "Thư viện PKCS#11 chỉ hỗ trợ Apple Silicon (ARM64). \
                Vui lòng liên hệ nhà cung cấp CA để xin phiên bản Intel (x86_64)."
                .to_string()
        } else {
            format!(
                "Thư viện '{}' không tương thích với kiến trúc hệ thống. \
                Vui lòng liên hệ nhà cung cấp CA.",
                library_path
            )
        };

        ESignError::LibraryArchitectureMismatch {
            library_arch,
            host_arch,
            library_path: library_path.to_string(),
            guidance,
        }
    }

    /// Parse architecture info from dlopen error message
    fn parse_arch_from_error(error_str: &str) -> (String, String) {
        // Pattern: "have 'x86_64', need 'arm64e' or 'arm64'"
        let library_arch = if let Some(start) = error_str.find("have '") {
            let rest = &error_str[start + 6..];
            if let Some(end) = rest.find('\'') {
                rest[..end].to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        let host_arch = if let Some(start) = error_str.find("need '") {
            let rest = &error_str[start + 6..];
            if let Some(end) = rest.find('\'') {
                rest[..end].to_string()
            } else {
                std::env::consts::ARCH.to_string()
            }
        } else {
            std::env::consts::ARCH.to_string()
        };

        (library_arch, host_arch)
    }

    /// Validate library path is in allowed locations (security measure)
    /// Prevents arbitrary code injection via malicious PKCS#11 libraries
    fn validate_library_path(path: &str) -> Result<(), ESignError> {
        // Define allowed prefixes per platform (hardcoded for security)
        #[cfg(target_os = "macos")]
        let allowed_prefixes: &[&str] = &["/Library/", "/usr/local/lib/"];
        #[cfg(target_os = "windows")]
        let allowed_prefixes: &[&str] = &["C:\\Program Files\\", "C:\\Program Files (x86)\\"];
        #[cfg(target_os = "linux")]
        let allowed_prefixes: &[&str] = &["/usr/lib/", "/usr/local/lib/", "/opt/"];
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        let allowed_prefixes: &[&str] = &["/usr/lib/"];

        // Resolve to canonical path to prevent path traversal
        let path_canonical = std::fs::canonicalize(path)
            .map_err(|e| ESignError::Pkcs11(format!("Invalid library path '{}': {}", path, e)))?;
        let path_str = path_canonical.to_string_lossy();

        // Check if path starts with any allowed prefix
        if !allowed_prefixes.iter().any(|p| path_str.starts_with(p)) {
            return Err(ESignError::Pkcs11(format!(
                "Library path '{}' not in allowed location. Allowed: {:?}",
                path_str, allowed_prefixes
            )));
        }

        // Verify file extension matches expected library format
        #[cfg(target_os = "macos")]
        let valid_ext = path_str.ends_with(".dylib");
        #[cfg(target_os = "windows")]
        let valid_ext = path_str.ends_with(".dll");
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let valid_ext = path_str.ends_with(".so");

        if !valid_ext {
            return Err(ESignError::Pkcs11(format!(
                "Library path '{}' has invalid extension",
                path_str
            )));
        }

        Ok(())
    }

    /// Create new TokenManager with specified PKCS#11 library path
    /// Validates library path against allowed locations before loading
    pub fn new(library_path: &str) -> Result<Self, ESignError> {
        // Validate library path is in allowed location (security check)
        Self::validate_library_path(library_path)?;

        // Load PKCS#11 library
        let ctx = Pkcs11::new(library_path).map_err(|e| {
            let error_str = e.to_string();

            // Detect architecture mismatch on macOS
            // Pattern: "incompatible architecture (have 'x86_64', need 'arm64e' or 'arm64')"
            if error_str.contains("incompatible architecture") {
                return Self::create_arch_mismatch_error(&error_str, library_path);
            }

            ESignError::Pkcs11(format!(
                "Failed to load PKCS#11 library '{}': {}",
                library_path, e
            ))
        })?;

        // Initialize the library
        ctx.initialize(CInitializeArgs::OsThreads)
            .map_err(|e| ESignError::Pkcs11(format!("Failed to initialize PKCS#11: {}", e)))?;

        Ok(Self {
            ctx,
            session: Mutex::new(None),
            signing_key: Mutex::new(None),
            certificate_der: Mutex::new(None),
            library_path: library_path.to_string(),
        })
    }

    /// Auto-detect available PKCS#11 libraries
    /// Returns list of detected libraries with CA names
    pub fn auto_detect() -> Vec<DetectedLibrary> {
        library_paths::all_paths()
            .into_iter()
            .filter(|(_, path)| std::path::Path::new(path).exists())
            .map(|(name, path)| DetectedLibrary {
                ca_name: name.to_string(),
                path: path.to_string(),
            })
            .collect()
    }

    /// Get library path
    pub fn library_path(&self) -> &str {
        &self.library_path
    }

    /// List available token slots
    pub fn list_slots(&self) -> Result<Vec<TokenInfo>, ESignError> {
        let slots = self
            .ctx
            .get_slots_with_token()
            .map_err(|e| ESignError::Pkcs11(format!("Failed to enumerate slots: {}", e)))?;

        let mut tokens = Vec::new();
        for slot in slots {
            match self.get_token_info(slot) {
                Ok(info) => tokens.push(info),
                Err(e) => {
                    // Log but continue with other slots
                    eprintln!(
                        "Warning: Failed to get token info for slot {:?}: {}",
                        slot, e
                    );
                }
            }
        }

        Ok(tokens)
    }

    /// Get token information for a specific slot
    fn get_token_info(&self, slot: Slot) -> Result<TokenInfo, ESignError> {
        let token_info = self
            .ctx
            .get_token_info(slot)
            .map_err(|e| ESignError::Pkcs11(format!("Failed to get token info: {}", e)))?;

        // Convert fixed-size label to trimmed string
        let label = token_info.label().trim().to_string();
        let manufacturer = token_info.manufacturer_id().trim().to_string();
        let model = token_info.model().trim().to_string();
        let serial = token_info.serial_number().trim().to_string();

        Ok(TokenInfo {
            slot_id: slot.id(),
            label,
            manufacturer,
            model,
            serial,
            has_token: true,
        })
    }

    /// Login to token with PIN
    /// Opens a session and authenticates with user PIN
    /// PIN is securely zeroized after authentication attempt
    pub fn login(&self, slot_id: u64, pin: &str) -> Result<(), ESignError> {
        // Find the slot
        let slots = self
            .ctx
            .get_slots_with_token()
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::TokenNotFound,
                message: format!("Failed to get slots: {}", e),
            })?;

        let slot = slots
            .into_iter()
            .find(|s| s.id() == slot_id)
            .ok_or_else(|| ESignError::Signing {
                code: SigningErrorCode::TokenNotFound,
                message: format!("Slot {} not found", slot_id),
            })?;

        // Open a read-write session
        let session = self
            .ctx
            .open_rw_session(slot)
            .map_err(|e| ESignError::Pkcs11(format!("Failed to open session: {}", e)))?;

        // Login with user PIN - create mutable copy for zeroization
        let mut pin_copy = pin.to_string();
        let auth_pin = AuthPin::new(pin_copy.clone());
        let login_result = session.login(UserType::User, Some(&auth_pin));

        // Securely zeroize PIN copy immediately after use
        pin_copy.zeroize();

        // Handle login result after zeroization
        login_result.map_err(|e| ESignError::Signing {
            code: SigningErrorCode::SigningFailed,
            message: format!("PIN authentication failed: {}", e),
        })?;

        // Find signing private key
        let key_handle = self.find_signing_key(&session)?;

        // Find and store certificate
        let cert_der = self.find_certificate(&session)?;

        // Store session, key handle, and certificate
        {
            let mut session_guard = self
                .session
                .lock()
                .map_err(|_| ESignError::Pkcs11("Session mutex poisoned".to_string()))?;
            *session_guard = Some(session);
        }
        {
            let mut key_guard = self
                .signing_key
                .lock()
                .map_err(|_| ESignError::Pkcs11("Signing key mutex poisoned".to_string()))?;
            *key_guard = Some(key_handle);
        }
        {
            let mut cert_guard = self
                .certificate_der
                .lock()
                .map_err(|_| ESignError::Pkcs11("Certificate mutex poisoned".to_string()))?;
            *cert_guard = Some(cert_der);
        }

        Ok(())
    }

    /// Find private key with signing capability
    fn find_signing_key(&self, session: &Session) -> Result<ObjectHandle, ESignError> {
        // Search for private key objects
        let template = vec![
            Attribute::Class(ObjectClass::PRIVATE_KEY),
            Attribute::Sign(true),
        ];

        let objects = session
            .find_objects(&template)
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::PrivateKeyNotFound,
                message: format!("Failed to search for private key: {}", e),
            })?;

        objects
            .into_iter()
            .next()
            .ok_or_else(|| ESignError::Signing {
                code: SigningErrorCode::PrivateKeyNotFound,
                message: "No signing private key found on token".to_string(),
            })
    }

    /// Find certificate on token
    fn find_certificate(&self, session: &Session) -> Result<Vec<u8>, ESignError> {
        // Search for certificate objects
        let template = vec![Attribute::Class(ObjectClass::CERTIFICATE)];

        let objects = session
            .find_objects(&template)
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: format!("Failed to search for certificate: {}", e),
            })?;

        let cert_handle = objects
            .into_iter()
            .next()
            .ok_or_else(|| ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: "No certificate found on token".to_string(),
            })?;

        // Get certificate value (DER-encoded)
        let attrs = session
            .get_attributes(cert_handle, &[AttributeType::Value])
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: format!("Failed to read certificate: {}", e),
            })?;

        for attr in attrs {
            if let Attribute::Value(der) = attr {
                return Ok(der);
            }
        }

        Err(ESignError::Signing {
            code: SigningErrorCode::CertificateNotFound,
            message: "Certificate value not found".to_string(),
        })
    }

    /// Get certificate information from logged-in token
    pub fn get_certificate_info(&self) -> Result<CertificateInfo, ESignError> {
        let cert_der = {
            let guard = self
                .certificate_der
                .lock()
                .map_err(|_| ESignError::Pkcs11("Certificate mutex poisoned".to_string()))?;
            guard.clone().ok_or_else(|| ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: "Not logged in or no certificate available".to_string(),
            })?
        };

        // Parse certificate with x509-parser
        let (_, cert) = X509Certificate::from_der(&cert_der).map_err(|e| ESignError::Signing {
            code: SigningErrorCode::CertificateNotFound,
            message: format!("Failed to parse certificate: {}", e),
        })?;

        // Extract certificate fields
        let serial = cert.serial.to_string();
        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();

        // Format dates as Vietnamese standard: dd/MM/yyyy HH:mm:ss
        let valid_from = format_datetime(cert.validity().not_before.timestamp());
        let valid_to = format_datetime(cert.validity().not_after.timestamp());

        // Calculate SHA-256 thumbprint
        let mut hasher = Sha256::new();
        hasher.update(&cert_der);
        let thumbprint = hex::encode(hasher.finalize());

        // Base64 encode the DER certificate
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let der_base64 = STANDARD.encode(&cert_der);

        Ok(CertificateInfo {
            serial,
            subject,
            issuer,
            valid_from,
            valid_to,
            thumbprint,
            der_base64,
        })
    }

    /// Get raw DER-encoded certificate bytes
    pub fn get_certificate_der(&self) -> Result<Vec<u8>, ESignError> {
        let guard = self
            .certificate_der
            .lock()
            .map_err(|_| ESignError::Pkcs11("Certificate mutex poisoned".to_string()))?;
        guard.clone().ok_or_else(|| ESignError::Signing {
            code: SigningErrorCode::CertificateNotFound,
            message: "Not logged in or no certificate available".to_string(),
        })
    }

    /// Sign data using RSA-PKCS#1 v1.5 with SHA-256
    /// Input: raw data to sign (mechanism handles hashing)
    /// Output: signature bytes
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, ESignError> {
        let session_guard = self
            .session
            .lock()
            .map_err(|_| ESignError::Pkcs11("Session mutex poisoned".to_string()))?;
        let session = session_guard.as_ref().ok_or_else(|| ESignError::Signing {
            code: SigningErrorCode::TokenNotFound,
            message: "Not logged in".to_string(),
        })?;

        let key_guard = self
            .signing_key
            .lock()
            .map_err(|_| ESignError::Pkcs11("Signing key mutex poisoned".to_string()))?;
        let key = key_guard.ok_or_else(|| ESignError::Signing {
            code: SigningErrorCode::PrivateKeyNotFound,
            message: "No signing key available".to_string(),
        })?;

        // Use Sha256RsaPkcs - mechanism handles hashing internally
        // DO NOT hash data manually to avoid double-hashing
        let mechanism = Mechanism::Sha256RsaPkcs;

        let signature = session
            .sign(&mechanism, key, data)
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::SigningFailed,
                message: format!("Signing operation failed: {}", e),
            })?;

        Ok(signature)
    }

    /// Sign pre-hashed data (digest) using RSA-PKCS#1 v1.5
    /// Input: SHA-256 digest with DigestInfo (typically 51 bytes for SHA-256)
    /// Output: signature bytes
    #[allow(dead_code)]
    pub fn sign_digest(&self, digest: &[u8]) -> Result<Vec<u8>, ESignError> {
        let session_guard = self
            .session
            .lock()
            .map_err(|_| ESignError::Pkcs11("Session mutex poisoned".to_string()))?;
        let session = session_guard.as_ref().ok_or_else(|| ESignError::Signing {
            code: SigningErrorCode::TokenNotFound,
            message: "Not logged in".to_string(),
        })?;

        let key_guard = self
            .signing_key
            .lock()
            .map_err(|_| ESignError::Pkcs11("Signing key mutex poisoned".to_string()))?;
        let key = key_guard.ok_or_else(|| ESignError::Signing {
            code: SigningErrorCode::PrivateKeyNotFound,
            message: "No signing key available".to_string(),
        })?;

        // Use RSA-PKCS for signing pre-computed digest
        // Caller must include DigestInfo structure for PKCS#1 v1.5
        let mechanism = Mechanism::RsaPkcs;

        let signature = session
            .sign(&mechanism, key, digest)
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::SigningFailed,
                message: format!("Signing digest failed: {}", e),
            })?;

        Ok(signature)
    }

    /// Logout and close session
    pub fn logout(&self) {
        // Clear stored handles - ignore poison errors during cleanup
        if let Ok(mut key_guard) = self.signing_key.lock() {
            *key_guard = None;
        }
        if let Ok(mut cert_guard) = self.certificate_der.lock() {
            *cert_guard = None;
        }
        if let Ok(mut session_guard) = self.session.lock() {
            if let Some(session) = session_guard.take() {
                // Attempt to logout, ignore errors
                let _ = session.logout();
                // Session is dropped here, closing it
            }
        }
    }

    /// Check if currently logged in
    pub fn is_logged_in(&self) -> bool {
        self.session.lock().map(|g| g.is_some()).unwrap_or(false)
    }
}

impl Drop for TokenManager {
    fn drop(&mut self) {
        self.logout();
        // Finalize PKCS#11 context
        // Note: cryptoki handles this in its own Drop
    }
}

/// Format Unix timestamp as Vietnamese datetime format
/// Format: HH:mm:ss dd/MM/yyyy (VNPT-CA standard)
fn format_datetime(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    let dt = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(Utc::now);
    dt.format("%H:%M:%S %d/%m/%Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ DetectedLibrary Tests ============

    #[test]
    fn test_detected_library_creation() {
        let lib = DetectedLibrary {
            ca_name: "VNPT-CA".to_string(),
            path: "/usr/local/lib/libVnptCaPlugin.dylib".to_string(),
        };
        assert_eq!(lib.ca_name, "VNPT-CA");
        assert!(lib.path.contains("Vnpt"));
    }

    #[test]
    fn test_detected_library_serialize() {
        let lib = DetectedLibrary {
            ca_name: "Test".to_string(),
            path: "/test/path".to_string(),
        };
        let json = serde_json::to_string(&lib).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("/test/path"));
    }

    // ============ TokenInfo Tests ============

    #[test]
    fn test_token_info_creation() {
        let info = TokenInfo {
            slot_id: 0,
            label: "Test Token".to_string(),
            manufacturer: "Test Manufacturer".to_string(),
            model: "Test Model".to_string(),
            serial: "123456".to_string(),
            has_token: true,
        };
        assert_eq!(info.slot_id, 0);
        assert!(info.has_token);
        assert_eq!(info.label, "Test Token");
    }

    #[test]
    fn test_token_info_without_token() {
        let info = TokenInfo {
            slot_id: 1,
            label: "Empty Slot".to_string(),
            manufacturer: String::new(),
            model: String::new(),
            serial: String::new(),
            has_token: false,
        };
        assert!(!info.has_token);
        assert!(info.serial.is_empty());
    }

    #[test]
    fn test_token_info_serialize() {
        let info = TokenInfo {
            slot_id: 42,
            label: "Token".to_string(),
            manufacturer: "Mfg".to_string(),
            model: "Model".to_string(),
            serial: "SN123".to_string(),
            has_token: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("SN123"));
    }

    // ============ CertificateInfo Tests ============

    #[test]
    fn test_certificate_info_creation() {
        let cert = CertificateInfo {
            serial: "ABC123".to_string(),
            subject: "CN=Test User".to_string(),
            issuer: "CN=Test CA".to_string(),
            valid_from: "2025-01-01".to_string(),
            valid_to: "2026-01-01".to_string(),
            thumbprint: "AABBCCDD".to_string(),
            der_base64: "BASE64DATA".to_string(),
        };
        assert_eq!(cert.serial, "ABC123");
        assert!(cert.subject.contains("Test User"));
        assert!(cert.issuer.contains("Test CA"));
    }

    #[test]
    fn test_certificate_info_serialize() {
        let cert = CertificateInfo {
            serial: "123".to_string(),
            subject: "CN=User".to_string(),
            issuer: "CN=CA".to_string(),
            valid_from: "2025-01-01".to_string(),
            valid_to: "2026-01-01".to_string(),
            thumbprint: "THUMB".to_string(),
            der_base64: "DATA".to_string(),
        };
        let json = serde_json::to_string(&cert).unwrap();
        assert!(json.contains("serial"));
        assert!(json.contains("thumbprint"));
    }

    // ============ Library Paths Tests ============

    #[test]
    fn test_vnpt_library_path() {
        let path = library_paths::vnpt::PATH;
        assert!(!path.is_empty());
        // Path should contain platform-specific extension
        #[cfg(target_os = "macos")]
        assert!(path.contains("dylib"));
        #[cfg(target_os = "windows")]
        assert!(path.contains("dll"));
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        assert!(path.contains("so"));
    }

    #[test]
    fn test_viettel_library_path() {
        let path = library_paths::viettel::PATH;
        assert!(!path.is_empty());
    }

    #[test]
    fn test_fpt_library_path() {
        let path = library_paths::fpt::PATH;
        assert!(!path.is_empty());
    }

    #[test]
    fn test_all_paths() {
        let paths = library_paths::all_paths();
        assert_eq!(paths.len(), 3); // VNPT, Viettel, FPT
        for (name, path) in paths {
            assert!(!name.is_empty());
            assert!(!path.is_empty());
        }
    }

    // ============ Auto Detect Tests ============

    #[test]
    fn test_auto_detect_returns_empty_when_no_libraries() {
        // This test just ensures auto_detect doesn't panic
        let detected = TokenManager::auto_detect();
        // May or may not find libraries depending on system
        assert!(detected.len() <= 3);
    }

    #[test]
    fn test_auto_detect_returns_valid_structure() {
        let detected = TokenManager::auto_detect();
        for lib in detected {
            assert!(!lib.ca_name.is_empty());
            assert!(!lib.path.is_empty());
        }
    }

    // ============ Format Datetime Tests ============

    #[test]
    fn test_format_datetime() {
        // Test a known timestamp
        let formatted = format_datetime(0);
        assert!(formatted.contains("/"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_format_datetime_recent() {
        // Test a recent timestamp (2025-01-01 00:00:00 UTC)
        let timestamp = 1735689600;
        let formatted = format_datetime(timestamp);
        assert!(formatted.contains("2025"));
    }

    #[test]
    fn test_format_datetime_format() {
        let formatted = format_datetime(0);
        // Format should be HH:mm:ss dd/MM/yyyy
        let parts: Vec<&str> = formatted.split(' ').collect();
        assert_eq!(parts.len(), 2);
        // Time part should have colons
        assert!(parts[0].contains(":"));
        // Date part should have slashes
        assert!(parts[1].contains("/"));
    }

    // ============ Token Manager Error Cases ============

    #[test]
    fn test_token_manager_invalid_path() {
        let result = TokenManager::new("/nonexistent/path/to/library.so");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_manager_empty_path() {
        let result = TokenManager::new("");
        assert!(result.is_err());
    }

    // ============ Serialization Round Trip Tests ============

    #[test]
    fn test_detected_library_roundtrip() {
        let original = DetectedLibrary {
            ca_name: "VNPT-CA".to_string(),
            path: "/path/to/lib".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: DetectedLibrary = serde_json::from_str(&json).unwrap();
        assert_eq!(original.ca_name, restored.ca_name);
        assert_eq!(original.path, restored.path);
    }

    #[test]
    fn test_token_info_roundtrip() {
        let original = TokenInfo {
            slot_id: 5,
            label: "My Token".to_string(),
            manufacturer: "Maker".to_string(),
            model: "Model X".to_string(),
            serial: "SN999".to_string(),
            has_token: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TokenInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(original.slot_id, restored.slot_id);
        assert_eq!(original.has_token, restored.has_token);
    }

    #[test]
    fn test_certificate_info_roundtrip() {
        let original = CertificateInfo {
            serial: "SER123".to_string(),
            subject: "CN=Test".to_string(),
            issuer: "CN=CA".to_string(),
            valid_from: "2025-01-01".to_string(),
            valid_to: "2026-12-31".to_string(),
            thumbprint: "ABCD1234".to_string(),
            der_base64: "dGVzdA==".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: CertificateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(original.serial, restored.serial);
        assert_eq!(original.der_base64, restored.der_base64);
    }

    // ============ Architecture Mismatch Detection Tests ============

    #[test]
    fn test_parse_arch_x86_to_arm64() {
        let error = "dlopen failed: mach-o file, but is an incompatible architecture (have 'x86_64', need 'arm64e' or 'arm64')";
        let (lib_arch, host_arch) = TokenManager::parse_arch_from_error(error);
        assert_eq!(lib_arch, "x86_64");
        assert_eq!(host_arch, "arm64e");
    }

    #[test]
    fn test_parse_arch_arm64_to_x86() {
        let error = "dlopen failed: incompatible architecture (have 'arm64', need 'x86_64')";
        let (lib_arch, host_arch) = TokenManager::parse_arch_from_error(error);
        assert_eq!(lib_arch, "arm64");
        assert_eq!(host_arch, "x86_64");
    }

    #[test]
    fn test_parse_arch_unknown_format() {
        let error = "some other error without architecture info";
        let (lib_arch, host_arch) = TokenManager::parse_arch_from_error(error);
        assert_eq!(lib_arch, "unknown");
        // Should fallback to std::env::consts::ARCH
        assert!(!host_arch.is_empty());
    }

    #[test]
    fn test_arch_mismatch_error_guidance_arm64_host() {
        let error = "incompatible architecture (have 'x86_64', need 'arm64')";
        let err = TokenManager::create_arch_mismatch_error(error, "/usr/local/lib/test.dylib");
        match err {
            crate::error::ESignError::LibraryArchitectureMismatch {
                library_arch,
                host_arch,
                guidance,
                ..
            } => {
                assert_eq!(library_arch, "x86_64");
                assert_eq!(host_arch, "arm64");
                assert!(guidance.contains("Rosetta"));
            }
            _ => panic!("Expected LibraryArchitectureMismatch error"),
        }
    }
}
