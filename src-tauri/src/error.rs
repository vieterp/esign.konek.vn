//! Error types for eSign Desktop
//!
//! Implements VNPT-CA compatible error codes (0-11)

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Signing error codes compatible with VNPT-CA Plugin
/// See docs/vnpt-ca-compatibility.md for full specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum SigningErrorCode {
    /// 0: Signing successful
    Success = 0,
    /// 1: Empty or invalid input data
    InvalidInput = 1,
    /// 2: Certificate not found
    CertificateNotFound = 2,
    /// 3: Signing operation failed
    SigningFailed = 3,
    /// 4: Private key not found on token
    PrivateKeyNotFound = 4,
    /// 5: Unknown/unspecified error
    UnknownError = 5,
    /// 6: Page parameter missing (PDF signing)
    PageParameterMissing = 6,
    /// 7: Invalid page number for signature placement
    InvalidSignaturePage = 7,
    /// 8: USB token/smart card not found
    TokenNotFound = 8,
    /// 9: Cannot reference token card ID
    TokenReferenceError = 9,
    /// 10: Input data contains invalid existing signatures
    InvalidExistingSignature = 10,
    /// 11: User cancelled the operation
    UserCancelled = 11,
}

/// Certificate validation error codes (VNPT-CA compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum CertValidationCode {
    /// 0: Certificate is valid
    Valid = 0,
    /// 1: Unknown validation error
    UnknownError = 1,
    /// 2: Certificate has expired
    Expired = 2,
    /// 3: Certificate not yet valid
    NotYetValid = 3,
    /// 4: Certificate has been revoked
    Revoked = 4,
    /// 5: Certificate cannot be used for signing
    CannotSign = 5,
    /// 6: Revocation check failed
    RevocationCheckFailed = 6,
    /// 7: Certificate not from trusted CA
    UntrustedCA = 7,
    /// 8: Cannot retrieve certificate info
    CertInfoUnavailable = 8,
    /// 9: Cannot retrieve CA certificate info
    CACertInfoUnavailable = 9,
    /// 10: OCSP server URL not found
    OCSPUrlNotFound = 10,
}

/// Main error type for eSign operations
#[derive(Error, Debug)]
pub enum ESignError {
    #[error("PKCS#11 error: {0}")]
    Pkcs11(String),

    #[error("Library architecture mismatch: {library_arch} library cannot run on {host_arch} system. {guidance}")]
    LibraryArchitectureMismatch {
        library_arch: String,
        host_arch: String,
        library_path: String,
        guidance: String,
    },

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("TSA error: {0}")]
    Tsa(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Signing error (code {code:?}): {message}")]
    Signing {
        code: SigningErrorCode,
        message: String,
    },

    #[error("Certificate validation error (code {code:?}): {message}")]
    #[allow(dead_code)]
    CertValidation {
        code: CertValidationCode,
        message: String,
    },
}

/// Result type for signing operations, compatible with VNPT-CA response format
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SigningResult {
    pub code: i32,
    pub data: String,
    pub error: String,
}

impl SigningResult {
    /// Create a success result
    #[allow(dead_code)]
    pub fn success(data: String) -> Self {
        Self {
            code: SigningErrorCode::Success as i32,
            data,
            error: String::new(),
        }
    }

    /// Create an error result
    #[allow(dead_code)]
    pub fn error(code: SigningErrorCode, message: &str) -> Self {
        Self {
            code: code as i32,
            data: String::new(),
            error: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ SigningErrorCode Tests ============

    #[test]
    fn test_signing_error_code_values() {
        assert_eq!(SigningErrorCode::Success as i32, 0);
        assert_eq!(SigningErrorCode::InvalidInput as i32, 1);
        assert_eq!(SigningErrorCode::CertificateNotFound as i32, 2);
        assert_eq!(SigningErrorCode::SigningFailed as i32, 3);
        assert_eq!(SigningErrorCode::PrivateKeyNotFound as i32, 4);
        assert_eq!(SigningErrorCode::UnknownError as i32, 5);
        assert_eq!(SigningErrorCode::PageParameterMissing as i32, 6);
        assert_eq!(SigningErrorCode::InvalidSignaturePage as i32, 7);
        assert_eq!(SigningErrorCode::TokenNotFound as i32, 8);
        assert_eq!(SigningErrorCode::TokenReferenceError as i32, 9);
        assert_eq!(SigningErrorCode::InvalidExistingSignature as i32, 10);
        assert_eq!(SigningErrorCode::UserCancelled as i32, 11);
    }

    #[test]
    fn test_signing_error_code_equality() {
        assert_eq!(SigningErrorCode::Success, SigningErrorCode::Success);
        assert_ne!(SigningErrorCode::Success, SigningErrorCode::InvalidInput);
    }

    #[test]
    fn test_signing_error_code_serialize() {
        let code = SigningErrorCode::TokenNotFound;
        let json = serde_json::to_string(&code).unwrap();
        // Enum serializes as string variant name by default
        assert!(json.contains("TokenNotFound"));
    }

    #[test]
    fn test_signing_error_code_deserialize() {
        let json = "\"TokenNotFound\"";
        let code: SigningErrorCode = serde_json::from_str(json).unwrap();
        assert_eq!(code, SigningErrorCode::TokenNotFound);
    }

    // ============ CertValidationCode Tests ============

    #[test]
    fn test_cert_validation_code_values() {
        assert_eq!(CertValidationCode::Valid as i32, 0);
        assert_eq!(CertValidationCode::UnknownError as i32, 1);
        assert_eq!(CertValidationCode::Expired as i32, 2);
        assert_eq!(CertValidationCode::NotYetValid as i32, 3);
        assert_eq!(CertValidationCode::Revoked as i32, 4);
        assert_eq!(CertValidationCode::CannotSign as i32, 5);
        assert_eq!(CertValidationCode::RevocationCheckFailed as i32, 6);
        assert_eq!(CertValidationCode::UntrustedCA as i32, 7);
        assert_eq!(CertValidationCode::CertInfoUnavailable as i32, 8);
        assert_eq!(CertValidationCode::CACertInfoUnavailable as i32, 9);
        assert_eq!(CertValidationCode::OCSPUrlNotFound as i32, 10);
    }

    #[test]
    fn test_cert_validation_code_equality() {
        assert_eq!(CertValidationCode::Valid, CertValidationCode::Valid);
        assert_ne!(CertValidationCode::Valid, CertValidationCode::Expired);
    }

    // ============ ESignError Tests ============

    #[test]
    fn test_esign_error_pkcs11() {
        let err = ESignError::Pkcs11("Token not found".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("PKCS#11"));
        assert!(msg.contains("Token not found"));
    }

    #[test]
    fn test_esign_error_pdf() {
        let err = ESignError::Pdf("Invalid PDF".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("PDF"));
        assert!(msg.contains("Invalid PDF"));
    }

    #[test]
    fn test_esign_error_tsa() {
        let err = ESignError::Tsa("Connection failed".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("TSA"));
        assert!(msg.contains("Connection failed"));
    }

    #[test]
    fn test_esign_error_signing() {
        let err = ESignError::Signing {
            code: SigningErrorCode::TokenNotFound,
            message: "USB Token not connected".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Signing error"));
        assert!(msg.contains("TokenNotFound"));
    }

    #[test]
    fn test_esign_error_debug() {
        let err = ESignError::Pkcs11("Test error".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Pkcs11"));
    }

    // ============ SigningResult Tests ============

    #[test]
    fn test_signing_result_success() {
        let result = SigningResult::success("base64data".to_string());
        assert_eq!(result.code, 0);
        assert_eq!(result.data, "base64data");
        assert!(result.error.is_empty());
    }

    #[test]
    fn test_signing_result_error() {
        let result = SigningResult::error(SigningErrorCode::TokenNotFound, "Token not connected");
        assert_eq!(result.code, 8);
        assert!(result.data.is_empty());
        assert_eq!(result.error, "Token not connected");
    }

    #[test]
    fn test_signing_result_serialize() {
        let result = SigningResult::success("test".to_string());
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"code\":0"));
        assert!(json.contains("\"data\":\"test\""));
    }

    #[test]
    fn test_signing_result_deserialize() {
        let json = r#"{"code":0,"data":"test","error":""}"#;
        let result: SigningResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.code, 0);
        assert_eq!(result.data, "test");
    }

    // ============ Error Code Ranges ============

    #[test]
    fn test_all_signing_error_codes_in_range() {
        // VNPT-CA spec: codes 0-11
        let codes = vec![
            SigningErrorCode::Success,
            SigningErrorCode::InvalidInput,
            SigningErrorCode::CertificateNotFound,
            SigningErrorCode::SigningFailed,
            SigningErrorCode::PrivateKeyNotFound,
            SigningErrorCode::UnknownError,
            SigningErrorCode::PageParameterMissing,
            SigningErrorCode::InvalidSignaturePage,
            SigningErrorCode::TokenNotFound,
            SigningErrorCode::TokenReferenceError,
            SigningErrorCode::InvalidExistingSignature,
            SigningErrorCode::UserCancelled,
        ];

        for code in codes {
            let value = code as i32;
            assert!(
                (0..=11).contains(&value),
                "Code {:?} = {} out of range",
                code,
                value
            );
        }
    }

    #[test]
    fn test_all_cert_validation_codes_in_range() {
        // VNPT-CA spec: codes 0-10
        let codes = vec![
            CertValidationCode::Valid,
            CertValidationCode::UnknownError,
            CertValidationCode::Expired,
            CertValidationCode::NotYetValid,
            CertValidationCode::Revoked,
            CertValidationCode::CannotSign,
            CertValidationCode::RevocationCheckFailed,
            CertValidationCode::UntrustedCA,
            CertValidationCode::CertInfoUnavailable,
            CertValidationCode::CACertInfoUnavailable,
            CertValidationCode::OCSPUrlNotFound,
        ];

        for code in codes {
            let value = code as i32;
            assert!(
                (0..=10).contains(&value),
                "Code {:?} = {} out of range",
                code,
                value
            );
        }
    }
}
