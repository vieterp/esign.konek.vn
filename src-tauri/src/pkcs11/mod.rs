//! PKCS#11 USB Token Communication Module
//!
//! Handles communication with Vietnamese CA USB tokens (VNPT, Viettel, FPT)
//! using the PKCS#11 standard via the cryptoki crate.

pub mod helpers;
pub mod library_paths;
mod manager;
mod types;

#[cfg(test)]
mod tests;

// Re-export public types
pub use manager::TokenManager;
pub use types::{CertificateInfo, DetectedLibrary, TokenInfo};
