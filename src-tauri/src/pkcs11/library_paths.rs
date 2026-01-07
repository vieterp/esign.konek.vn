//! Known PKCS#11 library paths for Vietnamese CAs
//!
//! Defines platform-specific paths for VNPT, Viettel, FPT, and OpenSC libraries.

/// All known library paths for auto-detection
pub fn all_paths() -> Vec<(&'static str, &'static str)> {
    vec![
        ("VNPT-CA", vnpt::PATH),
        ("Viettel-CA", viettel::PATH),
        ("FPT-CA", fpt::PATH),
        ("OpenSC (Generic PKCS#11)", opensc::PATH),
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
    pub const PATH: &str = "/usr/local/lib/viettel-ca_v6.dylib";
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

/// OpenSC PKCS#11 library paths (supports ePass2003, Feitian, and other generic tokens)
pub mod opensc {
    #[cfg(target_os = "macos")]
    pub const PATH: &str = "/usr/local/lib/opensc-pkcs11.so";
    #[cfg(target_os = "windows")]
    pub const PATH: &str = "C:\\Program Files\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub const PATH: &str = "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so";
}
