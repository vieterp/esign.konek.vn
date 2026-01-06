//! TokenManager - PKCS#11 token operations
//!
//! Thread-safe wrapper around cryptoki session for USB token communication.

use crate::error::{ESignError, SigningErrorCode};
use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType, ObjectClass, ObjectHandle},
    session::{Session, UserType},
    slot::Slot,
    types::AuthPin,
};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use x509_parser::prelude::*;
use zeroize::Zeroize;

use super::helpers::{create_arch_mismatch_error, format_dn_utf8, validate_library_path};
use super::library_paths;
use super::types::{format_datetime, CertificateInfo, DetectedLibrary, TokenInfo};

/// Token manager - handles PKCS#11 operations
/// Thread-safe wrapper around cryptoki session
pub struct TokenManager {
    ctx: Pkcs11,
    session: Mutex<Option<Session>>,
    signing_key: Mutex<Option<ObjectHandle>>,
    certificate_der: Mutex<Option<Vec<u8>>>,
    /// Full certificate chain (end-entity + issuers)
    certificate_chain: Mutex<Vec<Vec<u8>>>,
    library_path: String,
}

impl TokenManager {
    /// Create new TokenManager with specified PKCS#11 library path
    /// Validates library path against allowed locations before loading
    pub fn new(library_path: &str) -> Result<Self, ESignError> {
        // Validate library path is in allowed location (security check)
        validate_library_path(library_path)?;

        // Load PKCS#11 library
        let ctx = Pkcs11::new(library_path).map_err(|e| {
            let error_str = e.to_string();

            // Detect architecture mismatch on macOS
            if error_str.contains("incompatible architecture") {
                return create_arch_mismatch_error(&error_str, library_path);
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
            certificate_chain: Mutex::new(Vec::new()),
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

        if slots.is_empty() {
            return Ok(Vec::new());
        }

        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        for slot in &slots {
            match self.get_token_info(*slot) {
                Ok(info) => tokens.push(info),
                Err(e) => {
                    let error_msg = format!("Slot {}: {}", slot.id(), e);
                    errors.push(error_msg);
                }
            }
        }

        // If all slots failed, return an error instead of empty array
        if tokens.is_empty() && !errors.is_empty() {
            return Err(ESignError::Pkcs11(format!(
                "Found {} slot(s) with token but failed to read token info:\n{}",
                slots.len(),
                errors.join("\n")
            )));
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

        // Find certificate chain (end-entity + issuers)
        let (cert_der, cert_chain) = self.find_certificate_chain(&session)?;

        // Log chain info
        if cert_chain.len() > 1 {
            eprintln!(
                "Found certificate chain with {} certificates",
                cert_chain.len()
            );
        } else {
            eprintln!("Found single certificate (no issuer chain on token)");
        }

        // Store session, key handle, certificate, and chain
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
        {
            let mut chain_guard = self
                .certificate_chain
                .lock()
                .map_err(|_| ESignError::Pkcs11("Certificate chain mutex poisoned".to_string()))?;
            *chain_guard = cert_chain;
        }

        Ok(())
    }

    /// Find private key with signing capability
    fn find_signing_key(&self, session: &Session) -> Result<ObjectHandle, ESignError> {
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

    /// Find all certificates on token and build certificate chain
    /// Returns (end_entity_cert, full_chain) where chain is ordered [end_entity, issuer1, issuer2, ...]
    fn find_certificate_chain(
        &self,
        session: &Session,
    ) -> Result<(Vec<u8>, Vec<Vec<u8>>), ESignError> {
        let template = vec![Attribute::Class(ObjectClass::CERTIFICATE)];

        let objects = session
            .find_objects(&template)
            .map_err(|e| ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: format!("Failed to search for certificates: {}", e),
            })?;

        if objects.is_empty() {
            return Err(ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: "No certificates found on token".to_string(),
            });
        }

        // Extract all certificate DER values
        let mut all_certs: Vec<Vec<u8>> = Vec::new();
        for cert_handle in objects {
            let attrs = session
                .get_attributes(cert_handle, &[AttributeType::Value])
                .map_err(|e| ESignError::Signing {
                    code: SigningErrorCode::CertificateNotFound,
                    message: format!("Failed to read certificate: {}", e),
                })?;

            for attr in attrs {
                if let Attribute::Value(der) = attr {
                    all_certs.push(der);
                    break;
                }
            }
        }

        if all_certs.is_empty() {
            return Err(ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: "No certificate values found".to_string(),
            });
        }

        // Find end-entity certificate (the one with a matching private key)
        // For simplicity, use the first certificate as end-entity
        let end_entity = all_certs[0].clone();

        // Build chain by matching subject/issuer
        let chain = self.build_certificate_chain(&end_entity, &all_certs);

        Ok((end_entity, chain))
    }

    /// Build certificate chain from subject/issuer matching
    /// Returns ordered chain: [end_entity, issuer1, issuer2, ...]
    fn build_certificate_chain(&self, end_entity: &[u8], all_certs: &[Vec<u8>]) -> Vec<Vec<u8>> {
        use x509_parser::prelude::*;

        let mut chain = vec![end_entity.to_vec()];
        let mut current_cert = end_entity;

        // Maximum chain length to prevent infinite loops
        const MAX_CHAIN_LENGTH: usize = 10;

        for _ in 0..MAX_CHAIN_LENGTH {
            // Parse current certificate to get issuer
            let Ok((_, cert)) = X509Certificate::from_der(current_cert) else {
                break;
            };

            let issuer = cert.issuer();
            let subject = cert.subject();

            // If self-signed (issuer == subject), we've reached the root
            if issuer == subject {
                break;
            }

            // Find issuer certificate
            let mut found_issuer = false;
            for candidate in all_certs {
                if candidate == current_cert {
                    continue;
                }

                let Ok((_, cand_cert)) = X509Certificate::from_der(candidate) else {
                    continue;
                };

                // Check if candidate's subject matches current cert's issuer
                if cand_cert.subject() == issuer {
                    chain.push(candidate.clone());
                    current_cert = chain.last().unwrap();
                    found_issuer = true;
                    break;
                }
            }

            if !found_issuer {
                // No issuer found on token - chain is incomplete but still usable
                break;
            }
        }

        chain
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
        let subject = format_dn_utf8(cert.subject());
        let issuer = format_dn_utf8(cert.issuer());

        // Format dates as Vietnamese standard
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

    /// Get full certificate chain (end-entity + issuers)
    /// Returns Vec of DER-encoded certificates ordered [end_entity, issuer1, issuer2, ...]
    /// May return single certificate if no issuer chain found on token
    #[allow(dead_code)] // Ready for PAdES-LT/LTA integration
    pub fn get_certificate_chain(&self) -> Result<Vec<Vec<u8>>, ESignError> {
        let guard = self
            .certificate_chain
            .lock()
            .map_err(|_| ESignError::Pkcs11("Certificate chain mutex poisoned".to_string()))?;
        if guard.is_empty() {
            return Err(ESignError::Signing {
                code: SigningErrorCode::CertificateNotFound,
                message: "Not logged in or no certificate chain available".to_string(),
            });
        }
        Ok(guard.clone())
    }

    /// Sign data using RSA-PKCS#1 v1.5 with SHA-256
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
        if let Ok(mut chain_guard) = self.certificate_chain.lock() {
            chain_guard.clear();
        }
        if let Ok(mut session_guard) = self.session.lock() {
            if let Some(session) = session_guard.take() {
                let _ = session.logout();
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
    }
}
