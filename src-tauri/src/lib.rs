//! eSign Desktop - Cross-platform PDF signing with Vietnamese USB tokens
//!
//! This library provides the backend functionality for the eSign Desktop application,
//! including PKCS#11 token communication, PDF signing, and TSA integration.

mod error;
mod pdf;
mod pkcs11;
mod tsa;

use pdf::{PdfSigner, PdfSigningEngine, SignResult};
use pkcs11::{CertificateInfo, DetectedLibrary, TokenInfo, TokenManager};
use std::sync::Mutex;
use tauri::State;

/// Application state shared across commands
/// Uses Mutex for thread-safe access to TokenManager
pub struct AppState {
    token_manager: Mutex<Option<TokenManager>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            token_manager: Mutex::new(None),
        }
    }
}

/// Tauri command: Get application info
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "eSign Desktop",
        "version": "0.1.2",
        "description": "Cross-platform PDF signing with Vietnamese USB tokens"
    })
}

/// Tauri command: Detect available PKCS#11 libraries
/// Returns list of detected CA libraries (VNPT, Viettel, FPT)
#[tauri::command]
fn detect_libraries() -> Vec<DetectedLibrary> {
    TokenManager::auto_detect()
}

/// Tauri command: Initialize token manager with specified library
/// Must be called before other token operations
#[tauri::command]
fn init_token_manager(state: State<AppState>, library_path: String) -> Result<(), String> {
    // Drop old manager first to ensure C_Finalize is called
    {
        let mut guard = state
            .token_manager
            .lock()
            .map_err(|_| "Token manager mutex poisoned")?;

        if let Some(old_manager) = guard.take() {
            // Check if re-initializing with same library (skip if identical)
            if old_manager.library_path() == library_path {
                *guard = Some(old_manager);
                return Ok(());
            }
            // Explicit drop triggers C_Finalize via Drop impl
            drop(old_manager);
        }
    } // guard released here

    // Delay to ensure PKCS#11 library fully finalized
    // cryptoki v0.7.0's finalize() consumes self, so we rely on Drop cleanup + delay
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Create new manager
    let manager = TokenManager::new(&library_path).map_err(|e| e.to_string())?;

    let mut guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    *guard = Some(manager);

    Ok(())
}

/// Tauri command: List available tokens/slots
#[tauri::command]
fn list_tokens(state: State<AppState>) -> Result<Vec<TokenInfo>, String> {
    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    let manager = guard
        .as_ref()
        .ok_or("Token manager not initialized. Call init_token_manager first.")?;

    manager.list_slots().map_err(|e| e.to_string())
}

/// Tauri command: Login to token with PIN
#[tauri::command]
fn login_token(state: State<AppState>, slot_id: u64, pin: String) -> Result<(), String> {
    // Validate PIN length (4-16 characters)
    if pin.len() < 4 || pin.len() > 16 {
        return Err("PIN must be 4-16 characters".into());
    }

    // Validate PIN contains only alphanumeric characters
    if !pin.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("PIN contains invalid characters".into());
    }

    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    let manager = guard.as_ref().ok_or("Token manager not initialized")?;

    manager.login(slot_id, &pin).map_err(|e| e.to_string())
}

/// Tauri command: Get certificate information from logged-in token
#[tauri::command]
fn get_certificate(state: State<AppState>) -> Result<CertificateInfo, String> {
    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    let manager = guard.as_ref().ok_or("Token manager not initialized")?;

    manager.get_certificate_info().map_err(|e| e.to_string())
}

/// Tauri command: Logout from token
#[tauri::command]
fn logout_token(state: State<AppState>) -> Result<(), String> {
    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    if let Some(manager) = guard.as_ref() {
        manager.logout();
    }
    Ok(())
}

/// Tauri command: Check token status
/// Returns connection status and certificate info if logged in
#[tauri::command]
fn check_token_status(state: State<AppState>) -> Result<serde_json::Value, String> {
    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;

    match guard.as_ref() {
        Some(manager) => {
            let logged_in = manager.is_logged_in();
            let cert_info = if logged_in {
                manager.get_certificate_info().ok()
            } else {
                None
            };

            Ok(serde_json::json!({
                "initialized": true,
                "logged_in": logged_in,
                "library_path": manager.library_path(),
                "certificate": cert_info
            }))
        }
        None => {
            // Check for available libraries
            let detected = TokenManager::auto_detect();
            Ok(serde_json::json!({
                "initialized": false,
                "logged_in": false,
                "detected_libraries": detected
            }))
        }
    }
}

/// Tauri command: Sign data using token
/// Input: base64-encoded data to sign
/// Output: base64-encoded signature
#[tauri::command]
fn sign_data(state: State<AppState>, data_base64: String) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    let manager = guard.as_ref().ok_or("Token manager not initialized")?;

    // Decode input data
    let data = STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Invalid base64 input: {}", e))?;

    // Sign the data
    let signature = manager.sign(&data).map_err(|e| e.to_string())?;

    // Encode signature as base64
    Ok(STANDARD.encode(&signature))
}

/// Tauri command: Sign a PDF file
/// Requires token to be logged in first
#[tauri::command]
fn sign_pdf(
    state: State<AppState>,
    pdf_path: String,
    output_path: String,
    visible: bool,
    reason: Option<String>,
    signer_name: Option<String>,
    page: Option<u32>,
) -> Result<SignResult, String> {
    // Validate paths are not empty
    if pdf_path.is_empty() || output_path.is_empty() {
        return Err("Paths cannot be empty".into());
    }

    // Validate page number (1-1000 range)
    if let Some(p) = page {
        if p == 0 || p > 1000 {
            return Err("Invalid page number (must be 1-1000)".into());
        }
    }

    // Validate reason length
    if let Some(ref r) = reason {
        if r.len() > 500 {
            return Err("Reason too long (max 500 characters)".into());
        }
    }

    // Validate signer name length
    if let Some(ref s) = signer_name {
        if s.len() > 200 {
            return Err("Signer name too long (max 200 characters)".into());
        }
    }

    let guard = state
        .token_manager
        .lock()
        .map_err(|_| "Token manager mutex poisoned")?;
    let manager = guard
        .as_ref()
        .ok_or("Token manager not initialized. Call init_token_manager first.")?;

    if !manager.is_logged_in() {
        return Err("Not logged in. Call login_token first.".to_string());
    }

    // Get certificate from token
    let cert_der = manager.get_certificate_der().map_err(|e| e.to_string())?;
    let cert_info = manager.get_certificate_info().map_err(|e| e.to_string())?;

    // Build signer parameters
    let final_signer = signer_name.or_else(|| Some(cert_info.subject.clone()));

    let signer_params = PdfSigner {
        page: page.unwrap_or(1),
        llx: 50.0,
        lly: 50.0,
        urx: 250.0,
        ury: 100.0,
        visible,
        description: reason,
        signer: final_signer,
        signing_time: Some(pdf::get_current_signing_time()),
        certificate_serial: Some(cert_info.serial.clone()),
        ..Default::default()
    };

    // Create signing engine without TSA (Vietnamese TSA servers are unreliable)
    // Signatures will be valid but won't have trusted timestamps
    let engine = PdfSigningEngine::new();

    // Sign the PDF
    // Create a closure that captures manager for signing
    let sign_fn = |data: &[u8]| manager.sign(data);

    engine
        .sign_pdf(&pdf_path, &output_path, &signer_params, sign_fn, &cert_der)
        .map_err(|e| e.to_string())
}

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .setup(|_app| {
            // DevTools: Uncomment to auto-open in debug mode
            // #[cfg(debug_assertions)]
            // {
            //     let window = _app.get_webview_window("main").unwrap();
            //     window.open_devtools();
            // }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            detect_libraries,
            init_token_manager,
            list_tokens,
            login_token,
            get_certificate,
            logout_token,
            check_token_status,
            sign_data,
            sign_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
