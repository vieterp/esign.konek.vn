//! PKCS#11 helper functions
//!
//! Contains certificate parsing helpers, path validation, and architecture detection.

use crate::error::ESignError;
use x509_parser::prelude::*;

/// Format X.509 Distinguished Name with proper UTF-8 support
/// Handles Vietnamese characters that x509_parser's default to_string() corrupts
pub fn format_dn_utf8(name: &x509_parser::x509::X509Name) -> String {
    use x509_parser::der_parser::asn1_rs::Any;

    let mut parts = Vec::new();

    for rdn in name.iter() {
        for attr in rdn.iter() {
            // Get attribute type (CN, L, O, etc.)
            let oid_string = attr.attr_type().to_id_string();
            let attr_type = match oid_string.as_str() {
                "2.5.4.3" => "CN",
                "2.5.4.6" => "C",
                "2.5.4.7" => "L",
                "2.5.4.8" => "ST",
                "2.5.4.10" => "O",
                "2.5.4.11" => "OU",
                _ => &oid_string,
            };

            // Try to decode value as UTF-8 string
            let value = if let Ok((_rest, any)) = Any::from_der(attr.attr_value().as_bytes()) {
                // UTF8String (tag 12) and PrintableString (tag 19) both use UTF-8
                if any.tag().0 == 12 || any.tag().0 == 19 {
                    String::from_utf8_lossy(any.data).to_string()
                }
                // Try BMPString (tag 30) - UTF-16BE encoding
                else if any.tag().0 == 30 {
                    // BMPString is UTF-16BE
                    let utf16_chars: Vec<u16> = any.data
                        .chunks_exact(2)
                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                        .collect();
                    String::from_utf16(&utf16_chars).unwrap_or_else(|_| {
                        String::from_utf8_lossy(any.data).to_string()
                    })
                }
                // Fallback to default
                else {
                    attr.as_str().unwrap_or("?").to_string()
                }
            } else {
                attr.as_str().unwrap_or("?").to_string()
            };

            parts.push(format!("{}={}", attr_type, value));
        }
    }

    parts.join(", ")
}

/// Validate library path is in allowed locations (security measure)
/// Prevents arbitrary code injection via malicious PKCS#11 libraries
pub fn validate_library_path(path: &str) -> Result<(), ESignError> {
    // Define allowed prefixes per platform (hardcoded for security)
    #[cfg(target_os = "macos")]
    let allowed_prefixes: &[&str] = &["/Library/", "/usr/local/lib/"];
    #[cfg(target_os = "windows")]
    let allowed_prefixes: &[&str] = &[
        "C:\\Program Files\\",
        "C:\\Program Files (x86)\\",
        // Vietnamese CA standard installation paths
        "C:\\vnpt-ca\\",
        "C:\\Viettel-CA\\",
        "C:\\FPT-CA\\",
    ];
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

/// Create architecture mismatch error with helpful guidance
/// Parses the error message to extract architecture info and provides actionable advice
pub fn create_arch_mismatch_error(error_str: &str, library_path: &str) -> ESignError {
    // Parse architectures from error: "have 'x86_64', need 'arm64e' or 'arm64'"
    let (library_arch, host_arch) = parse_arch_from_error(error_str);

    // Generate platform-specific guidance
    let guidance = if host_arch.contains("arm64") && library_arch.contains("x86_64") {
        // Apple Silicon Mac with x86_64 library
        "Thư viện PKCS#11 của nhà cung cấp chỉ hỗ trợ Intel (x86_64). \
            Giải pháp: (1) Liên hệ nhà cung cấp CA để xin phiên bản ARM64, \
            hoặc (2) Chạy ứng dụng qua Rosetta 2: arch -x86_64 open -a \"Konek eSign\"".to_string()
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
pub fn parse_arch_from_error(error_str: &str) -> (String, String) {
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
