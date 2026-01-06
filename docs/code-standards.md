# Code Standards and Implementation Guidelines

**Last Updated:** 2026-01-06
**Version:** 1.1

## Table of Contents

1. [Naming Conventions](#naming-conventions)
2. [TypeScript/Frontend Standards](#typescriptfrontend-standards)
3. [Rust/Backend Standards](#rustbackend-standards)
4. [Project Structure](#project-structure)
5. [Error Handling](#error-handling)
6. [Testing Strategy](#testing-strategy)
7. [Documentation](#documentation)
8. [Security Best Practices](#security-best-practices)

---

## Naming Conventions

### TypeScript/JavaScript

#### PascalCase
- React component files: `SigningButton.tsx`, `PdfPreview.tsx`
- Type/Interface names: `SigningResult`, `TokenInfo`, `CertificateInfo`
- Enum names: `SigningErrorCode`, `TokenStatus`

```typescript
// ✓ Correct
export interface PdfSigningOptions {
  page: number;
  coordinates: SignatureCoordinates;
}

// ✗ Incorrect
export interface pdf_signing_options {}
```

#### camelCase
- Variable names: `tokenStatus`, `isConnected`, `certificateSerial`
- Function names: `checkTokenStatus()`, `signPdfDocument()`
- Object properties: `llx`, `lly`, `urx`, `ury`

```typescript
// ✓ Correct
const tokenDetected = checkToken();
function handleSigningComplete() {}
const pdfSigner: PdfSigner = { page: 1, llx: 100, ... };

// ✗ Incorrect
const TokenDetected = checkToken();
function Handle_Signing_Complete() {}
```

#### SCREAMING_SNAKE_CASE
- Constants: `DEFAULT_PAGE_NUMBER`, `TIMEOUT_MS`, `TSA_URL`

```typescript
// ✓ Correct
const DEFAULT_SIGNATURE_SIZE = 10;
const AUTHORIZED_TSA_URLS = ['https://tsa.vpost.vn', ...];

// ✗ Incorrect
const defaultSignatureSize = 10;
const authorizedTsaUrls = [...];
```

### Rust

#### snake_case
- Variable names: `token_manager`, `pdf_signer`, `error_code`
- Function names: `sign_pdf()`, `detect_pkcs11_library()`, `validate_certificate()`
- Module names: `pkcs11`, `pdf`, `tsa`, `error`

```rust
// ✓ Correct
fn sign_pdf_document(pdf_data: &[u8]) -> Result<Vec<u8>, ESignError> {}
let token_manager = TokenManager::new(library_path)?;

// ✗ Incorrect
fn signPdfDocument(pdf_data: &[u8]) {} // camelCase not idiomatic
let TokenManager = ...; // not a constant
```

#### PascalCase
- Struct/Enum names: `TokenManager`, `CertificateInfo`, `SigningErrorCode`
- Trait names: `TokenOperations`, `PdfSigner`

```rust
// ✓ Correct
pub struct TokenManager { ... }
pub enum SigningErrorCode { ... }
pub trait PdfSigningOperation { ... }

// ✗ Incorrect
pub struct token_manager { ... }
pub enum signing_error_code { ... }
```

#### SCREAMING_SNAKE_CASE
- Constants: `DEFAULT_PIN_RETRIES`, `MAX_SIGNATURE_SIZE`, `TSA_TIMEOUT`

```rust
// ✓ Correct
const DEFAULT_PIN_RETRIES: u8 = 3;
const PKCS11_LIBRARY_TIMEOUT_MS: u64 = 5000;
```

---

## TypeScript/Frontend Standards

### Directory Structure

```
src/
├── components/          # Reusable UI components
│   ├── Button.tsx
│   ├── TokenStatus.tsx
│   ├── PdfPreview.tsx
│   └── SigningForm.tsx
├── pages/              # Page-level components (Phase 4)
│   ├── HomePage.tsx
│   ├── SigningPage.tsx
│   └── SettingsPage.tsx
├── hooks/              # Custom React hooks
│   ├── useToken.ts
│   ├── usePdfSignature.ts
│   └── useErrorHandler.ts
├── types/              # TypeScript type definitions
│   ├── index.ts
│   └── tauri.ts
├── utils/              # Utility functions
│   ├── formatting.ts
│   ├── validation.ts
│   └── tauri-helpers.ts
├── App.tsx             # Root component
├── main.tsx            # React DOM mount point
└── index.css           # Global styles
```

### Component Structure

```typescript
// ✓ Correct structure
import { ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Props interface
interface SigningButtonProps {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
  isLoading?: boolean;
  disabled?: boolean;
}

// Component
export function SigningButton({
  onSuccess,
  onError,
  isLoading = false,
  disabled = false,
}: SigningButtonProps) {
  const handleClick = async () => {
    try {
      const result = await invoke('sign_pdf');
      onSuccess?.();
    } catch (error) {
      onError?.(error as Error);
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled || isLoading}
      className="bg-primary-600 text-white"
    >
      {isLoading ? 'Signing...' : 'Sign PDF'}
    </button>
  );
}
```

### Imports

- Use absolute imports where configured
- Group imports: React → external libs → local types → local components
- One import per line for clarity

```typescript
// ✓ Correct
import { useState, useEffect } from 'react';

import { invoke } from '@tauri-apps/api/core';
import { readBinaryFile } from '@tauri-apps/api/fs';

import type { SigningResult } from '@/types';
import { PdfPreview } from '@/components/PdfPreview';
import { TokenStatus } from '@/components/TokenStatus';

// ✗ Incorrect (mixed order, multiple imports per line)
import { invoke, readBinaryFile } from '@tauri-apps/api';
import { useState } from 'react';
import { SigningResult } from '@/types';
```

### Event Handlers

Name with `handle` prefix for event handlers, `on` prefix for callbacks:

```typescript
// ✓ Correct
const handleFileSelect = (event: ChangeEvent<HTMLInputElement>) => { ... };
const handleSubmit = (event: FormEvent) => { ... };
const onSigningComplete = () => { ... };
const onError = (error: Error) => { ... };

// ✗ Incorrect
const fileSelect = () => { ... };
const fileSelectHandler = () => { ... };
const signComplete = () => { ... };
```

### Tailwind CSS Classes

- Use utility-first approach
- Group responsive variants together
- Use dark mode modifiers (`dark:`)
- Keep class lists readable (max 3-4 lines)

```tsx
// ✓ Correct
<div className="
  w-full max-w-md mx-auto
  px-4 py-6 md:px-6 md:py-8
  bg-white dark:bg-neutral-900
  rounded-lg shadow-md
  border border-neutral-200 dark:border-neutral-700
">
  Content
</div>

// Split complex styles
const buttonClasses = "
  px-4 py-2
  bg-primary-600 hover:bg-primary-700
  text-white
  rounded-md
  transition-colors
  disabled:opacity-50 disabled:cursor-not-allowed
";
```

### Type Safety

Always use strict TypeScript:

```typescript
// tsconfig.json settings (already configured)
{
  "compilerOptions": {
    "strict": true,              // All strict checks enabled
    "noUnusedLocals": true,      // Error on unused variables
    "noUnusedParameters": true,  // Error on unused params
    "noImplicitAny": true        // Error on implicit any
  }
}

// ✓ Correct - explicit types
function processResult(result: SigningResult): void {
  if (result.code === 0) {
    console.log(result.data);
  }
}

// ✗ Incorrect - implicit any
function processResult(result) {}
```

---

## Rust/Backend Standards

### Directory Structure

```
src-tauri/src/
├── main.rs        # Tauri app entry point
├── lib.rs         # Library exports, commands, AppState
├── error.rs       # Error types, VNPT-CA compatible codes
├── pkcs11.rs      # Token operations, library paths
├── pdf.rs         # PDF signing, manipulation
├── tsa.rs         # Timestamp Authority integration
└── (modules in Phase 2-3)
```

### Module Organization

Each module should have:
1. Module documentation
2. Public interface (types, traits, functions)
3. Private implementation details
4. Tests (at bottom or `#[cfg(test)]` module)

```rust
//! Token operations using PKCS#11
//!
//! Handles USB token communication via the cryptoki crate.
//! Supports VNPT, Viettel, and FPT CA tokens.
//!
//! # Example
//! ```ignore
//! let mut manager = TokenManager::new("/path/to/library")?;
//! manager.login(slot_id, "1234")?;
//! let cert = manager.get_certificate()?;
//! ```

use crate::error::{ESignError, SigningErrorCode};

/// Token information structure
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub slot_id: u64,
    pub label: String,
}

/// Public interface
impl TokenManager {
    pub fn new(library_path: &str) -> Result<Self, ESignError> {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        // Test code
    }
}
```

### Error Handling

VNPT-CA Compatible Error Codes (implemented in `error.rs`):

```rust
#[repr(i32)]
pub enum SigningErrorCode {
    Success = 0,                      // Ký thành công
    InvalidInput = 1,                 // Dữ liệu đầu vào rỗng/sai định dạng
    CertificateNotFound = 2,          // Không tìm thấy chứng thư số
    SigningFailed = 3,                // Ký thất bại
    PrivateKeyNotFound = 4,           // Không tìm thấy private key
    UnknownError = 5,                 // Nguyên nhân không xác định
    PageParameterMissing = 6,         // Thiếu tham số trang (PDF)
    InvalidSignaturePage = 7,         // Trang đặt chữ ký không hợp lệ
    TokenNotFound = 8,                // Không tìm thấy thẻ ký số
    TokenReferenceError = 9,          // Không tham chiếu được tới thẻ
    InvalidExistingSignature = 10,    // Dữ liệu chứa chữ ký không hợp lệ
    UserCancelled = 11,               // Người dùng hủy bỏ
}

// Use in Result types
pub type SigningResult = Result<Vec<u8>, ESignError>;

// Error conversion
impl From<std::io::Error> for ESignError {
    fn from(err: std::io::Error) -> Self {
        ESignError::Io(err)
    }
}
```

### Tauri Commands

Commands expose Rust functionality to frontend:

```rust
/// Sign PDF document (VNPT-CA compatible)
///
/// # Arguments
/// * `pdf_data` - Base64 encoded PDF
/// * `signer_params` - PdfSigner configuration
///
/// # Returns
/// SigningResult with code 0 on success
#[tauri::command]
async fn sign_pdf(
    pdf_data: String,
    signer_params: PdfSigner,
) -> Result<SigningResult, String> {
    // Implementation
    Ok(SigningResult::success(signed_data))
}

// Register in lib.rs
.invoke_handler(tauri::generate_handler![
    sign_pdf,
    check_token_status,
    // ... other commands
])
```

### Async/Await

Use `tokio` runtime (already configured):

```rust
// ✓ Correct - async command
#[tauri::command]
async fn sign_pdf(data: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        // Heavy computation in thread pool
    }).await.map_err(|e| e.to_string())
}

// ✓ Correct - async helper
pub async fn fetch_timestamp(data: &[u8]) -> Result<Vec<u8>, ESignError> {
    let response = reqwest::Client::new()
        .post(TSA_URL)
        .body(data.to_vec())
        .send()
        .await?;
    Ok(response.bytes().await?.to_vec())
}
```

### Documentation Comments

Use `///` for public items, `//!` for module docs:

```rust
/// Sign data using token's private key
///
/// # Arguments
/// * `data` - Raw data to sign (will be hashed)
/// * `hash_algorithm` - Algorithm to use (SHA-256, SHA-512)
///
/// # Returns
/// PKCS#7 signature bytes
///
/// # Errors
/// Returns `ESignError::Signing` if private key not found
///
/// # Example
/// ```ignore
/// let signature = manager.sign_data(pdf_bytes, HashAlgorithm::SHA256)?;
/// ```
pub fn sign_data(&self, data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>, ESignError> {
    // Implementation
}
```

---

## Project Structure

### Configuration Files Location

```
esign.konek.vn/
├── package.json              # npm dependencies, scripts
├── package-lock.json         # npm lock file
├── tsconfig.json             # TypeScript compiler (frontend)
├── tsconfig.node.json        # TypeScript config (build tools)
├── vite.config.ts            # Vite bundler config
├── tailwind.config.js        # Tailwind CSS config
├── postcss.config.js         # PostCSS config
├── index.html                # HTML entry point
├── src-tauri/
│   ├── Cargo.toml            # Rust dependencies
│   ├── Cargo.lock            # Rust lock file (must commit)
│   ├── tauri.conf.json       # Tauri app config
│   └── build.rs              # Tauri build script
└── .gitignore                # VCS exclusions
```

### Build Output

```
esign.konek.vn/
├── dist/                     # Frontend build (Vite output)
│   ├── index.html
│   ├── assets/
│   │   ├── index-{hash}.js
│   │   └── index-{hash}.css
│   └── ...
├── src-tauri/target/         # Rust build output
│   ├── debug/               # Debug binaries
│   └── release/             # Release binaries
└── node_modules/             # npm packages (excluded from git)
```

---

## Error Handling

### Frontend Error Handling

```typescript
// ✓ Correct error handling pattern
async function handlePdfSigning(pdfPath: string) {
  try {
    setIsLoading(true);
    const result = await invoke<SigningResult>('sign_pdf', { pdfPath });

    if (result.code !== 0) {
      setError(`Signing failed: ${result.error} (code: ${result.code})`);
      return;
    }

    setSignedData(result.data);
    onSuccess?.();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    setError(`Unexpected error: ${message}`);
    console.error('Signing error:', error);
  } finally {
    setIsLoading(false);
  }
}

// ✗ Incorrect (swallowing errors)
async function handlePdfSigning(pdfPath: string) {
  const result = await invoke('sign_pdf', { pdfPath });
  setSignedData(result.data); // No error checking!
}
```

### Rust Error Handling

```rust
// ✓ Correct - propagate with context
pub fn sign_pdf(pdf: &[u8]) -> Result<Vec<u8>, ESignError> {
    let hash = compute_sha256(pdf)
        .map_err(|e| ESignError::Pdf(format!("Hash computation failed: {}", e)))?;

    sign_hash(&hash)
        .map_err(|e| ESignError::Signing {
            code: SigningErrorCode::SigningFailed,
            message: format!("Private key operation failed: {}", e)
        })
}

// ✓ Use match for variant handling
match token_manager.login(pin) {
    Ok(_) => println!("Login successful"),
    Err(ESignError::Signing { code, message }) => {
        eprintln!("Signing error {}: {}", code as i32, message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}

// ✗ Incorrect - use unwrap only in tests/main
let result = risky_operation().unwrap(); // Panics on error!
```

---

## Testing Strategy

### Frontend Testing (Phase 5)

```typescript
// Using Vitest + React Testing Library
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { SigningButton } from '@/components/SigningButton';

describe('SigningButton', () => {
  it('should render button with correct text', () => {
    render(<SigningButton />);
    expect(screen.getByText('Sign PDF')).toBeInTheDocument();
  });

  it('should be disabled while signing', () => {
    render(<SigningButton isLoading={true} />);
    expect(screen.getByRole('button')).toBeDisabled();
  });
});
```

### Rust Testing (Phase 5)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_serialization() {
        let code = SigningErrorCode::Success;
        assert_eq!(code as i32, 0);
    }

    #[tokio::test]
    async fn test_signing_result() {
        let result = SigningResult::success("data".to_string());
        assert_eq!(result.code, 0);
        assert_eq!(result.error, "");
    }

    #[test]
    #[should_panic(expected = "token not found")]
    fn test_token_error() {
        let _ = TokenManager::new("invalid_path").unwrap();
    }
}
```

---

## Documentation

### Code Comments

Use comments sparingly (code should be self-documenting):

```typescript
// ✓ Explains "why" not "what"
// Token detection requires polling due to USB device latency
const pollInterval = setInterval(checkToken, 500);

// ✗ Obvious - redundant comment
const x = 5; // Set x to 5
```

### Documentation Files

- **README**: Project overview and quick start
- **CONTRIBUTING**: Development workflow
- **Architecture**: System design and component relationships
- **Code Standards** (this file): Naming, patterns, conventions
- **API docs**: Generated from code comments (Phase 5)

### Comments in Code

```rust
// Doc comments (///) for public APIs
/// Detects available PKCS#11 libraries for Vietnamese CAs
///
/// This function scans standard installation paths and returns
/// the first available library. The search order is:
/// 1. VNPT-CA
/// 2. Viettel-CA
/// 3. FPT-CA
///
/// # Returns
/// Path to available library or None if no token drivers found
pub fn auto_detect_pkcs11() -> Option<PathBuf> {
    // Implementation
}

// Regular comments for complex logic
// Cryptoki requires session before object enumeration
// See PKCS#11 spec section 6.3
let session = ctx.open_session(slot)?;
```

---

## Security Best Practices

### Phase 1 Security Model (Implemented)

**Content Security Policy (CSP) - Hardened**

The application enforces a strict CSP that prevents XSS and other injection attacks:

```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data:; connect-src 'self'"
```

Key restrictions:
- No `unsafe-inline` - Inline scripts/styles are not allowed
- `script-src 'self'` - Only bundled JavaScript can execute
- `connect-src 'self'` - No arbitrary network calls (TSA URLs added in Phase 3)
- External resources limited to Google Fonts (safe, non-executable)

**Filesystem Permissions - Scoped**

The application is restricted to Documents and Downloads directories:

```json
"fs:allow-read-file": [
  { "path": "$DOCUMENT/**" },
  { "path": "$DOWNLOAD/**" }
],
"fs:allow-write-file": [
  { "path": "$DOCUMENT/**" },
  { "path": "$DOWNLOAD/**" }
]
```

Benefits:
- Prevents access to system directories
- Blocks reading other user's files
- Restricts signed PDFs to user-accessible folders
- Conforms to OS security expectations

### Frontend Security

```typescript
// ✓ Always sanitize user input
function validatePdfPath(path: string): boolean {
  // Block directory traversal - filesystem scope already enforced by Tauri
  if (path.includes('..')) {
    return false;
  }
  return path.endsWith('.pdf');
}

// ✓ Use secure IPC (Tauri handles this)
// Tauri validates all invoke calls against capabilities/default.json
// No sensitive data allowed in localStorage/sessionStorage

// ✓ Inline styles must use CSS classes (CSP safe)
<div className="bg-primary-600 text-white">OK</div>  // ✓ Correct
<div style={{ background: 'red' }}>NOT OK</div>    // ✗ Violates CSP

// ✗ Never store sensitive data
// const pin = localStorage.getItem('token_pin'); // INSECURE!
// const privateKey = sessionStorage['key']; // INSECURE!
```

### Rust Security

```rust
// ✓ Validate all external input
pub fn validate_pin(pin: &str) -> Result<(), ESignError> {
    if pin.is_empty() {
        return Err(ESignError::Signing {
            code: SigningErrorCode::InvalidInput,
            message: "PIN cannot be empty".to_string(),
        });
    }
    if pin.len() > 12 {
        return Err(ESignError::Signing {
            code: SigningErrorCode::InvalidInput,
            message: "PIN too long".to_string(),
        });
    }
    Ok(())
}

// ✓ Use secure random number generation
use rand::thread_rng;
let nonce: [u8; 32] = thread_rng().gen();

// ✓ Clear sensitive data from memory
pub struct PinGuard {
    pin: Vec<u8>,
}

impl Drop for PinGuard {
    fn drop(&mut self) {
        // Overwrite with zeros
        for byte in &mut self.pin {
            *byte = 0;
        }
    }
}

// ✗ Never store PINs or keys in plain text
// let pin_stored = fs::write("pin.txt", pin)?; // INSECURE!
```

### PKCS#11 Security

```rust
// ✓ Validate library path before loading
pub fn load_pkcs11(library_path: &str) -> Result<Pkcs11, ESignError> {
    let path = Path::new(library_path);

    // Only allow known installation directories
    if !path.starts_with("/Library") && !path.starts_with("C:\\Program Files") {
        return Err(ESignError::Pkcs11("Invalid library path".to_string()));
    }

    // Load library
    Pkcs11::new(library_path)
        .map_err(|e| ESignError::Pkcs11(e.to_string()))
}
```

### CSP Compliance Guide

When adding new features, ensure CSP compliance:

```typescript
// ✗ VIOLATES CSP
<button onClick={() => eval('code')}>Click</button>  // Dynamic code
<div style={{ color: 'red' }}>Text</div>             // Inline styles
<img src="data:image/svg+xml" onload="alert('xss')"> // XSS vector
<script>alert('nope')</script>                       // Inline scripts

// ✓ CSP COMPLIANT
<button onClick={handleClick}>Click</button>         // Event handlers OK
<div className="text-red-500">Text</div>             // Use Tailwind classes
<img src="/images/icon.png" alt="icon" />            // Static images
import { dynamicModule } from '@/utils';             // Import modules
```

### Connect-src Restrictions (Phase 3)

Current CSP allows `connect-src 'self'` only. For Phase 3 TSA integration:

```json
"connect-src": "'self' https://tsa.vnca.com.vn https://tsa.vpost.vn https://tsa.fptca.com.vn"
```

The approved TSA endpoints are:
- `https://tsa.vnca.com.vn` - VNPT-CA TSA
- `https://tsa.vpost.vn` - VietPostCA TSA
- `https://tsa.fptca.com.vn` - FPT-CA TSA

Do not add arbitrary endpoints without security review.

---

## Code Review Checklist

Before committing code:

- [ ] Naming conventions followed (PascalCase, camelCase, SCREAMING_SNAKE_CASE)
- [ ] No `any` types in TypeScript (strict mode enabled)
- [ ] Error handling implemented (try/catch in TS, Result in Rust)
- [ ] No sensitive data logged or stored
- [ ] Code is self-documenting (minimal comments needed)
- [ ] Module structure follows standard layout
- [ ] Documentation/docstrings added for public APIs
- [ ] Tests written (Phase 5 onward)

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Tauri Documentation](https://tauri.app/v1/docs/)
- [VNPT-CA Compatibility](./vnpt-ca-compatibility.md)
