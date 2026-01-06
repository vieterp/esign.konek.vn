//! PKCS#11 module unit tests

use super::helpers::parse_arch_from_error;
use super::library_paths;
use super::manager::TokenManager;
use super::types::{format_datetime, CertificateInfo, DetectedLibrary, TokenInfo};

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
    assert_eq!(paths.len(), 4); // VNPT, Viettel, FPT, OpenSC
    for (name, path) in paths {
        assert!(!name.is_empty());
        assert!(!path.is_empty());
    }
}

// ============ Auto Detect Tests ============

#[test]
fn test_auto_detect_returns_empty_when_no_libraries() {
    let detected = TokenManager::auto_detect();
    assert!(detected.len() <= 4);
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
    let formatted = format_datetime(0);
    assert!(formatted.contains("/"));
    assert!(formatted.contains(":"));
}

#[test]
fn test_format_datetime_recent() {
    let timestamp = 1735689600;
    let formatted = format_datetime(timestamp);
    assert!(formatted.contains("2025"));
}

#[test]
fn test_format_datetime_format() {
    let formatted = format_datetime(0);
    let parts: Vec<&str> = formatted.split(' ').collect();
    assert_eq!(parts.len(), 2);
    assert!(parts[0].contains(":"));
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
    let (lib_arch, host_arch) = parse_arch_from_error(error);
    assert_eq!(lib_arch, "x86_64");
    assert_eq!(host_arch, "arm64e");
}

#[test]
fn test_parse_arch_arm64_to_x86() {
    let error = "dlopen failed: incompatible architecture (have 'arm64', need 'x86_64')";
    let (lib_arch, host_arch) = parse_arch_from_error(error);
    assert_eq!(lib_arch, "arm64");
    assert_eq!(host_arch, "x86_64");
}

#[test]
fn test_parse_arch_unknown_format() {
    let error = "some other error without architecture info";
    let (lib_arch, host_arch) = parse_arch_from_error(error);
    assert_eq!(lib_arch, "unknown");
    assert!(!host_arch.is_empty());
}

#[test]
fn test_arch_mismatch_error_guidance_arm64_host() {
    use super::helpers::create_arch_mismatch_error;
    let error = "incompatible architecture (have 'x86_64', need 'arm64')";
    let err = create_arch_mismatch_error(error, "/usr/local/lib/test.dylib");
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
