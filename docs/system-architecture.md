# System Architecture Documentation

**Last Updated:** 2026-01-06
**Version:** 1.2
**Status:** Phase 1 Security Complete - All major components implemented

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow](#data-flow)
4. [Technology Stack](#technology-stack)
5. [Module Responsibilities](#module-responsibilities)
6. [IPC Communication](#ipc-communication)
7. [Security Architecture](#security-architecture)
8. [Phase Progression](#phase-progression)

---

## Architecture Overview

### System Design Philosophy

eSign Desktop uses a **two-layer architecture**:

1. **Frontend (WebView)**: React + TypeScript for UI, delegating all cryptographic operations
2. **Backend (Native Rust)**: PKCS#11 operations, PDF signing, TSA integration

This separation ensures:
- **Security**: Private keys never exposed to JavaScript
- **Performance**: Native code for crypto operations
- **Maintainability**: Clear separation of concerns

### High-Level Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Environment                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Tauri Application Window                  │ │
│  │                                                        │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │            Frontend (WebView / Renderer)         │ │ │
│  │  │                                                  │ │ │
│  │  │  ┌─────────────────────────────────────────────┐ │ │ │
│  │  │  │  React Components (Phase 4)                 │ │ │ │
│  │  │  │  - SigningForm                              │ │ │ │
│  │  │  │  - TokenStatus                              │ │ │ │
│  │  │  │  - PdfPreview                               │ │ │ │
│  │  │  │  - ProgressIndicator                        │ │ │ │
│  │  │  └─────────────────────────────────────────────┘ │ │ │
│  │  │                      ▲                            │ │ │
│  │  │                      │ Tauri IPC (JSON)           │ │ │
│  │  │                      ▼                            │ │ │
│  │  │  ┌─────────────────────────────────────────────┐ │ │ │
│  │  │  │  Tauri API Layer (@tauri-apps/api)         │ │ │ │
│  │  │  │  - invoke()  ─► sign_pdf command          │ │ │ │
│  │  │  │  - listen()  ◄─ events from Rust          │ │ │ │
│  │  │  │  - fs APIs   ─► dialog operations         │ │ │ │
│  │  │  └─────────────────────────────────────────────┘ │ │ │
│  │  └──────────────────────────────────────────────────┘ │ │
│  │                                                        │ │
│  ├────── Tauri Bridge (IPC Communication) ───────────────┤ │
│  │                                                        │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │         Backend (Rust Native Code)              │ │ │
│  │  │                                                  │ │ │
│  │  │  ┌──────────────────────────────────────────┐   │ │ │
│  │  │  │  Tauri Commands (lib.rs)                │   │ │ │
│  │  │  │  - #[tauri::command] sign_pdf()         │   │ │ │
│  │  │  │  - #[tauri::command] check_token()      │   │ │ │
│  │  │  │  - (more in Phase 2-3)                  │   │ │ │
│  │  │  └──────────────────────────────────────────┘   │ │ │
│  │  │                      ▲                          │ │ │
│  │  │                      │ Function Calls           │ │ │
│  │  │                      ▼                          │ │ │
│  │  │  ┌──────────────────────────────────────────┐   │ │ │
│  │  │  │  Module Layer                           │   │ │ │
│  │  │  │                                          │   │ │ │
│  │  │  │  ┌────────────┐  ┌──────────┐  ┌──────┐│   │ │ │
│  │  │  │  │ pkcs11.rs  │  │ pdf.rs   │  │tsa.rs││   │ │ │
│  │  │  │  │ (Phase 2)  │  │(Phase 3) │  │(Ph 3)││   │ │ │
│  │  │  │  │            │  │          │  │      ││   │ │ │
│  │  │  │  │ Token ops  │  │PDF sign  │  │TSA   ││   │ │ │
│  │  │  │  └────────────┘  └──────────┘  └──────┘│   │ │ │
│  │  │  └──────────────────────────────────────────┘   │ │ │
│  │  │                      ▲                          │ │ │
│  │  │                      │ Library Calls             │ │ │
│  │  │                      ▼                          │ │ │
│  │  │  ┌──────────────────────────────────────────┐   │ │ │
│  │  │  │  External Crate Dependencies             │   │ │ │
│  │  │  │                                          │   │ │ │
│  │  │  │  ┌────────┐  ┌───────┐  ┌────────────┐  │   │ │ │
│  │  │  │  │cryptoki│  │ lopdf │  │ openssl-sys│  │   │ │ │
│  │  │  │  │PKCS#11 │  │  PDF  │  │  Crypto    │  │   │ │ │
│  │  │  │  └────────┘  └───────┘  └────────────┘  │   │ │ │
│  │  │  │  ┌────────┐  ┌───────────────────────┐  │   │ │ │
│  │  │  │  │ reqwest│  │ chrono, serde, etc.  │  │   │ │ │
│  │  │  │  │ HTTP   │  │ Utilities             │  │   │ │ │
│  │  │  │  └────────┘  └───────────────────────┘  │   │ │ │
│  │  │  └──────────────────────────────────────────┘   │ │ │
│  │  └──────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        System Resources                             │  │
│  │  - File System (Documents, Temp)                    │  │
│  │  - Network (TSA requests)                           │  │
│  │  - USB Devices (Token detection)                    │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ System Calls
                              ▼
        ┌─────────────────────────────────────┐
        │      Operating System (macOS/Windows)
        │                                      │
        │  ┌──────────────────────────────┐  │
        │  │  USB Device Interface         │  │
        │  │  - Token Detection           │  │
        │  │  - PKCS#11 Library Loading   │  │
        │  │  - Cryptographic Operations  │  │
        │  └──────────────────────────────┘  │
        │                                      │
        │  ┌──────────────────────────────┐  │
        │  │  Filesystem                  │  │
        │  │  - PDF Files                 │  │
        │  │  - Temp Directory            │  │
        │  └──────────────────────────────┘  │
        │                                      │
        │  ┌──────────────────────────────┐  │
        │  │  Network Stack               │  │
        │  │  - HTTPS to TSA              │  │
        │  │  - DNS Resolution            │  │
        │  └──────────────────────────────┘  │
        └─────────────────────────────────────┘
                      │
                      │ libvnpt_pkcs11.dylib
                      │ viettel-pkcs11.dll
                      │ fpt-pkcs11.dll
                      ▼
        ┌─────────────────────────────────────┐
        │   PKCS#11 Tokens (Hardware)         │
        │                                      │
        │  ┌──────────────────────────────┐  │
        │  │  USB Hardware Token           │  │
        │  │  - Private Key (protected)    │  │
        │  │  - X.509 Certificate          │  │
        │  │  - Token PIN                  │  │
        │  └──────────────────────────────┘  │
        └─────────────────────────────────────┘
```

---

## Component Architecture

### Phase 1: Foundation (Current)

**Status:** Complete

**Components Initialized:**
- Project structure
- Build toolchain (Tauri 2.x, Vite, TypeScript, Rust)
- Basic error type definitions (VNPT-CA compatible)
- Placeholder modules for Phases 2-3

**Key Files:**
- `src-tauri/src/error.rs` - Error codes 0-11, certification validation codes
- `src-tauri/src/pkcs11.rs` - Token detection, library paths
- `src-tauri/src/lib.rs` - Tauri command registration
- `src/App.tsx` - React root component

### Phase 2: PKCS#11 Integration (Next)

**Expected Deliverables:**
- Token auto-detection
- PIN entry dialog
- Certificate retrieval
- Private key operation interface

**New Modules:**
- Complete `pkcs11.rs` implementation
- Token credential management
- PKCS#11 session handling

### Phase 3: PDF Signing (Following)

**Expected Deliverables:**
- PDF parsing and manipulation
- Signature placeholder creation
- Hash computation
- TSA integration
- Signed PDF generation

**New Modules:**
- Complete `pdf.rs` implementation
- Complete `tsa.rs` implementation
- PAdES signature formatter

### Phase 4: UI Implementation (Subsequent)

**Expected Deliverables:**
- Complete React component set
- PDF preview
- Signing form with token status
- Progress indicator
- Error handling UI

**New Components:**
- `pages/SigningPage.tsx`
- `components/TokenStatus.tsx`
- `components/SigningForm.tsx`
- `components/PdfPreview.tsx`

### Phase 5: Testing & Distribution (Final)

**Expected Deliverables:**
- Unit and integration tests
- Code signing (macOS/Windows)
- Auto-update mechanism
- Distribution packages

---

## Data Flow

### PDF Signing Flow (Complete Flow)

```
1. User Interaction (Frontend)
   └─ Select PDF file
      └─ invoke('select_pdf') via dialog
         └─ Backend: Dialog::open_file()
            └─ User selects file: /Users/john/document.pdf

2. PDF Preview (Frontend)
   └─ invoke('load_pdf_preview', { path })
      └─ Backend: pdf::load_preview()
         └─ Read PDF file
         └─ Extract first page
         └─ Return as base64 image
      └─ Frontend: Display in <PdfPreview>

3. Token Detection (Frontend)
   └─ invoke('check_token_status')
      └─ Backend: pkcs11::TokenManager::auto_detect()
         └─ Scan PKCS#11 library paths
         └─ Try to initialize cryptoki
         └─ Return TokenInfo { connected: true, ... }
      └─ Frontend: Show "Token Ready"

4. PIN Entry (Frontend)
   └─ User enters PIN in dialog
      └─ Store in memory only (not localStorage!)

5. Signing Initiation (Frontend)
   └─ User clicks "Sign PDF" button
      └─ invoke('sign_pdf', {
           pdf_data: base64_encoded_pdf,
           pdf_signer: {
             page: 1,
             llx: 50, lly: 50, urx: 150, ury: 100,
             signer_name: "John Doe",
             signing_time: "14:30:00 26/12/2025"
           },
           pin: "1234"
         })

6. Backend Signing Operation
   └─ Rust: sign_pdf_command()
      ├─ Validate input
      ├─ Decode base64 PDF
      ├─ Parse PDF structure
      ├─ Create signature placeholder
      │  └─ Calculate byte range
      │  └─ Prepare PAdES signature dict
      ├─ Compute SHA-256 hash
      ├─ Connect to token
      │  └─ pkcs11::TokenManager::new()
      │  └─ Load cryptoki library
      │  └─ Open session
      │  └─ Login with PIN
      ├─ Sign hash with private key
      │  └─ cryptoki: C_SignInit(), C_Sign()
      │  └─ Returns PKCS#7 signature
      ├─ Request timestamp (if enabled)
      │  └─ reqwest: POST to TSA URL
      │  └─ Include signature + hash
      │  └─ Get timestamp token
      ├─ Embed signature + timestamp
      │  └─ Update PDF signature dict
      │  └─ Append signature bytes
      ├─ Write signed PDF
      │  └─ Return as base64
      └─ Return SigningResult {
           code: 0,
           data: base64_signed_pdf,
           error: ""
         }

7. Frontend Result Handling
   └─ Check code
      ├─ If code = 0: Success
      │  └─ Show success message
      │  └─ Offer "Save Signed PDF" dialog
      ├─ If code = 11: User cancelled
      │  └─ Show "Signing cancelled" message
      └─ If code > 0: Error
         └─ Show error dialog with code + message

8. Save Signed PDF (Frontend)
   └─ User clicks "Save PDF"
      └─ invoke('save_signed_pdf', {
           pdf_data: result.data,
           original_path: "/Users/john/document.pdf"
         })
      └─ Backend: fs::write("/Users/john/document_signed.pdf", pdf_data)
      └─ Frontend: Show "Saved successfully"
```

### Module Call Chain

```
Frontend Component
    └─ invoke('sign_pdf', data)
       └─ Tauri IPC Bridge
          └─ Rust: sign_pdf_command()
             └─ error::validate_input()
             ├─ pdf::parse_pdf()
             ├─ pdf::create_signature_placeholder()
             ├─ pdf::compute_sha256()
             │
             ├─ pkcs11::TokenManager::new()
             │  └─ cryptoki::Pkcs11::new()
             │
             ├─ TokenManager::login(pin)
             │  └─ cryptoki: C_OpenSession()
             │  └─ cryptoki: C_Login()
             │
             ├─ TokenManager::get_certificate()
             │  └─ cryptoki: C_FindObjects()
             │  └─ x509_parser: parse_x509()
             │
             ├─ TokenManager::sign(hash)
             │  └─ cryptoki: C_SignInit()
             │  └─ cryptoki: C_Sign()
             │  └─ Returns PKCS#7 signature
             │
             ├─ tsa::request_timestamp() [Optional]
             │  └─ reqwest: POST request
             │  └─ Parse RFC 3161 response
             │
             └─ pdf::embed_signature()
                └─ Returns base64 signed PDF
                └─ error::SigningResult::success()
                   └─ Returns to Frontend

Frontend
    └─ Receives result
       └─ Display success/error
```

---

## Technology Stack

### Frontend

| Layer | Technology | Version | Purpose |
|-------|-----------|---------|---------|
| **UI Framework** | React | 18.3.1 | Component-based UI |
| **Language** | TypeScript | 5.7.2 | Type safety |
| **Bundler** | Vite | 6.0.3 | Fast module bundling |
| **Styling** | Tailwind CSS | 3.4.16 | Utility-first CSS |
| **IPC** | Tauri API | 2.1.1 | Bridge to Rust backend |
| **PDF Viewing** | PDF.js | 4.0.0 | (Phase 4) Render PDFs |

### Backend

| Layer | Crate | Version | Purpose |
|-------|-------|---------|---------|
| **Framework** | Tauri | 2.x | Desktop app framework |
| **PKCS#11** | cryptoki | 0.7 | Token communication |
| **PDF** | lopdf | 0.34 | PDF manipulation |
| **Crypto** | sha2 | 0.10 | SHA-256 hashing |
| | x509-parser | 0.16 | Certificate parsing |
| **HTTP** | reqwest | 0.12 | TSA requests |
| **Serialization** | serde | 1.0 | JSON/data serialization |
| **Async** | tokio | 1.x | Async runtime |
| **Date/Time** | chrono | 0.4 | Date/time handling |
| **Error Handling** | thiserror | 1.0 | Error type derivation |

### Build & Development

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 18+ | JavaScript runtime |
| npm | 9+ | Package manager |
| Rust | 1.70+ | Backend compilation |
| Tauri CLI | 2.x | App building/packaging |
| PostCSS | 8.4.49 | CSS processing |
| Autoprefixer | 10.4.20 | CSS vendor prefixes |

---

## Module Responsibilities

### error.rs

**Responsibility:** Define all error types and VNPT-CA compatible error codes

**Exports:**
- `SigningErrorCode` enum (0-11) - Signing operation status
- `CertValidationCode` enum (0-10) - Certificate validation status
- `ESignError` enum - Internal error type
- `SigningResult` struct - API response format

**Usage:** All modules import and use these types for consistent error reporting

### pkcs11.rs (Phase 2)

**Responsibility:** USB token communication via PKCS#11

**To Implement:**
- Token detection (library auto-discovery)
- Session management
- PIN entry/validation
- Certificate retrieval
- Private key operations (signing)
- Token logout

**Exports:**
- `TokenManager` struct
- `TokenInfo` struct
- `CertificateInfo` struct
- `library_paths` module

**Dependencies:** cryptoki, chrono, error.rs

### pdf.rs (Phase 3)

**Responsibility:** PDF parsing and digital signature creation

**To Implement:**
- PDF file parsing
- Signature placeholder creation
- Byte range calculation
- Hash computation (SHA-256)
- PAdES signature formatting
- Signed PDF generation

**Exports:**
- `PdfSigner` struct
- `PdfSigningOps` trait

**Dependencies:** lopdf, sha2, x509-parser, error.rs

### tsa.rs (Phase 3)

**Responsibility:** Timestamp Authority integration (RFC 3161)

**To Implement:**
- TSA request formation
- RFC 3161 protocol
- Timestamp response parsing
- Timestamp embedding in PDF

**Exports:**
- `TsaClient` struct
- `TimestampResponse` struct

**Dependencies:** reqwest, serde, error.rs

### lib.rs

**Responsibility:** Tauri command registration and AppState

**Exports:**
- `run()` - Application entry point
- Tauri commands: `sign_pdf()`, `check_token_status()`, etc.

**Dependencies:** All modules (pkcs11, pdf, tsa, error)

---

## IPC Communication

### Tauri IPC Protocol

All communication uses JSON serialization over Tauri's custom protocol.

### Command Invocation (Frontend → Backend)

```typescript
// Type-safe invoke
import { invoke } from '@tauri-apps/api/core';

interface SignPdfPayload {
  pdf_data: string;        // base64
  pdf_signer: PdfSigner;   // coordinates, metadata
  pin: string;             // Token PIN
}

interface SigningResponse {
  code: number;            // 0-11 error code
  data: string;            // base64 signed PDF
  error: string;           // Error message
}

const result = await invoke<SigningResponse>('sign_pdf', {
  pdf_data,
  pdf_signer,
  pin,
});
```

### Tauri Command Definition (Backend)

```rust
#[tauri::command]
async fn sign_pdf(
    pdf_data: String,           // From IPC (base64)
    pdf_signer: PdfSigner,      // From IPC (serde)
    pin: String,
) -> Result<SigningResult, String> {
    // Implementation
    // Returns Result<SigningResult, String>
    // Tauri auto-serializes to JSON
}
```

### Event Emission (Backend → Frontend)

For long-running operations (Phase 4):

```rust
// Backend emits event
app_handle.emit_all("signing_progress", json!({
    "status": "computing_hash",
    "progress": 25,
})).ok();

// Frontend listens
import { listen } from '@tauri-apps/api/event';

listen('signing_progress', (event) => {
    console.log(event.payload); // { status, progress }
    setProgress(event.payload.progress);
});
```

---

## Security Architecture

### Key Security Principles

1. **Private Keys Never Leave Token**
   - All signing operations happen in hardware
   - Frontend cannot access private key
   - Backend uses PKCS#11 C_Sign() function

2. **PIN Entry Protection**
   - Entered directly in PIN dialog (Phase 2)
   - Not stored in browser storage
   - Cleared from memory after use
   - Not logged or transmitted insecurely

3. **Code Isolation**
   - Frontend: UI only, no crypto operations
   - Backend: All security-sensitive code
   - IPC bridge: No sensitive data in flight

4. **PKCS#11 Library Validation** (Phase 2)
   - Only load libraries from known paths
   - Validate library signatures (Windows)
   - Prevent DLL hijacking

### Capabilities and Permissions

**File:** `src-tauri/capabilities/default.json`

**Phase 1 Complete:** Scoped to Documents and Downloads
```json
{
  "permissions": [
    "core:default",
    "shell:allow-open",
    "dialog:allow-open",
    "dialog:allow-save",
    "dialog:allow-message",
    "dialog:allow-ask",
    {
      "identifier": "fs:allow-read-file",
      "allow": [
        { "path": "$DOCUMENT/**" },
        { "path": "$DOWNLOAD/**" }
      ]
    },
    {
      "identifier": "fs:allow-write-file",
      "allow": [
        { "path": "$DOCUMENT/**" },
        { "path": "$DOWNLOAD/**" }
      ]
    },
    {
      "identifier": "fs:allow-exists",
      "allow": [
        { "path": "$DOCUMENT/**" },
        { "path": "$DOWNLOAD/**" }
      ]
    }
  ]
}
```

**Security Model:** The application is restricted to Documents and Downloads folders on user's system. This prevents access to sensitive system directories or other user files.

### Content Security Policy (CSP)

**File:** `src-tauri/tauri.conf.json`

**Phase 1 Complete:** Hardened CSP without unsafe-inline
```json
{
  "security": {
    "csp": "default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self'"
  }
}
```

**Security Features:**
- `default-src 'self'` - Only allow resources from the application itself
- `script-src 'self'` - No inline scripts, all JS must be bundled
- `style-src 'self' https://fonts.googleapis.com` - Bundled CSS only, allow Google Fonts (safe external)
- `font-src 'self' https://fonts.gstatic.com` - Bundled fonts + Google Fonts CDN
- `img-src 'self' data:` - Application images + data URIs for embedded images
- `connect-src 'self'` - API calls only to self (no arbitrary network access)

This CSP prevents:
- Inline script execution (XSS mitigation)
- Dynamic eval() calls
- Arbitrary external script loading
- Unsafe CSS manipulation

---

## Phase Progression

### Phase 1: Foundation ✓ Complete
- Project initialization
- Build configuration
- Error type definitions
- Module structure
- **Phase 1 Security (Phase 1.1):** CSP hardening and FS scope restrictions
  - Content Security Policy without unsafe-inline
  - Filesystem access scoped to Documents + Downloads
  - PKCS#11 manager lifecycle improvements

### Phase 2: Token Integration
- PKCS#11 implementation
- Token detection
- Certificate retrieval
- PIN entry dialog

### Phase 3: PDF Signing
- PDF parsing
- Signature creation
- TSA integration
- PAdES formatting

### Phase 4: UI Implementation
- Complete React components
- PDF preview
- Signing workflow
- Error handling UI

### Phase 5: Testing & Distribution
- Unit tests
- Integration tests
- Code signing
- Auto-update
- Packaging

---

## Deployment Architecture

### Development

```
npm run tauri dev
├─ Vite dev server (port 1420)
├─ Hot module reload active
├─ Rust backend recompiles on change
└─ DevTools enabled
```

### Production Build

```
npm run tauri build
├─ npm run build (Vite production bundle)
│  └─ Minified JS/CSS, tree-shaked imports
├─ cargo build --release (Rust optimization)
│  └─ Release binary, no debug symbols
└─ Tauri bundler
   ├─ macOS: .dmg + .app (code signed)
   ├─ Windows: .msi + .exe (Authenticode signed)
   └─ Linux: .AppImage, .rpm, .deb
```

### Distribution

- **macOS:** Code sign + notarize (.dmg)
- **Windows:** Authenticode sign (.msi)
- **Auto-update:** Delta updates via tauri-plugin-updater

---

## References

- [Tauri 2.x Documentation](https://tauri.app/v2/)
- [cryptoki Documentation](https://docs.rs/cryptoki/latest/cryptoki/)
- [PKCS#11 Specification](https://oasis-tcs.github.io/pkcs11/)
- [PAdES Standard](https://www.etsi.org/deliver/etsi_ts/102700_102799/102778/01.04.02_60/ts_102778v010402p.pdf)
- [VNPT-CA Plugin Integration Guide](https://docs.vnpt.vn/ca/)
