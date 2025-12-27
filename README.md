# Konek eSign - Cross-Platform PDF Signing with Vietnamese USB Tokens

**Status:** v0.1.0 Released (Phase 4-5 Complete) | **Last Updated:** 2025-12-27
**Repository:** https://github.com/vieterp/esign.konek.vn | **Tauri ID:** vn.konek.esign

Konek eSign is a modern desktop application for digitally signing PDF documents using Vietnamese PKCS#11 USB security tokens (smart cards). Built with Tauri, React, TypeScript, and Rust, it provides a lightweight, secure alternative to browser-based signing solutions.

## Key Features

- **Cross-Platform:** macOS 12+ and Windows 10+ with native performance
- **USB Token Support:** VNPT-CA, Viettel-CA, FPT-CA integration via PKCS#11
- **PAdES-BES Signatures:** Legally compliant signatures per Decree 34/2019/NĐ-CP
- **RFC 3161 Timestamps:** Cryptographic timestamp authority (TSA) integration
- **Lightweight:** 18-25 MB binary (vs. Electron: 150-200 MB)
- **Dark Mode:** Full dark theme support via Tailwind CSS
- **Secure:** PIN zeroization, library path validation, no key exposure to frontend

## Quick Start

### Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+ stable
- **macOS** 12+ or **Windows** 10 (1909+)
- One of: VNPT-CA, Viettel-CA, or FPT-CA USB token

### Installation & Development

```bash
# Clone repository
git clone https://github.com/vieterp/esign.konek.vn.git
cd esign.konek.vn

# Install dependencies
npm install

# Start development server
npm run tauri dev

# Build production binaries
npm run tauri build
```

## Project Structure

```
esign.konek.vn/
├── src/                          # React frontend (TypeScript) - ~1,290 LOC
│   ├── components/               # 4 UI components (FileDropzone, TokenStatus, PinInput, ResultModal)
│   ├── hooks/                    # 2 custom hooks (useToken, useSigning)
│   ├── lib/                      # Tauri IPC bridge
│   ├── App.tsx                   # Main application component
│   └── index.css                 # Tailwind + global styles
├── src-tauri/                    # Rust backend - ~3,317 LOC
│   ├── src/
│   │   ├── lib.rs                # Tauri commands and AppState (260 lines)
│   │   ├── main.rs               # Application entry point
│   │   ├── error.rs              # VNPT-CA error codes (329 lines)
│   │   ├── pkcs11.rs             # Token operations (808 lines)
│   │   ├── pdf.rs                # PDF signing (1,302 lines)
│   │   └── tsa.rs                # TSA timestamp protocol (575 lines)
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── tauri.conf.json
├── docs/                         # Comprehensive documentation
│   ├── project-overview-pdr.md   # Requirements & PDR
│   ├── system-architecture.md    # Technical design
│   ├── code-standards.md         # Coding guidelines
│   ├── codebase-summary.md       # Codebase overview
│   ├── project-roadmap.md        # Release roadmap
│   └── tech-stack.md             # Dependency rationale
├── package.json
├── package-lock.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.js
```

## Architecture Overview

### Frontend (React + TypeScript)
- **Components:** FileDropzone, TokenStatus, PinInput, ResultModal
- **Hooks:** useToken (token state), useSigning (signing workflow)
- **IPC Bridge:** Type-safe Tauri command wrappers
- **UI Framework:** Tailwind CSS with custom palette

### Backend (Rust)
- **PKCS#11 Integration:** cryptoki crate for token communication
- **PDF Signing:** lopdf for PDF manipulation, PAdES-BES compliance
- **Timestamps:** RFC 3161 TSA with multiple endpoints (VNPT, Viettel, FPT)
- **Security:** SHA-256 hashing, RSA-PKCS#1 signatures, zeroized PINs
- **Tauri Commands:** 10+ IPC endpoints for frontend communication

### Supported Libraries

```rust
// macOS
"/Library/vnpt-ca/lib/libcryptoki.dylib"
"/Library/viettel-ca/libpkcs11.dylib"
"/Library/FPT/libpkcs11.dylib"

// Windows
"C:\\vnpt-ca\\cryptoki.dll"
"C:\\Viettel-CA\\pkcs11.dll"
"C:\\FPT-CA\\pkcs11.dll"
```

## Development Commands

```bash
# Development
npm run tauri dev          # Start dev server with Tauri window

# Building
npm run build              # TypeScript + Vite bundle
npm run tauri build        # Full Tauri app build (DMG/MSI)

# Code Quality
npx tsc --noEmit          # TypeScript type check
cargo clippy              # Rust linting
cargo audit               # Dependency vulnerability scan
npm audit                 # Frontend dependencies

# Testing (Phase 5)
npm test                  # Run test suite
cargo test                # Rust unit tests
```

## Workflow: Signing a PDF

1. **Token Detection** → Auto-detect CA library and available tokens
2. **PIN Entry** → Secure PIN dialog (zeroized after use)
3. **Certificate Display** → Show signer info and validity
4. **File Selection** → Drag-and-drop or browse PDF file
5. **PDF Signing** → Execute PAdES-BES signature with coordinates
6. **TSA Timestamp** → Add cryptographic timestamp (RFC 3161)
7. **Output Generation** → Save signed PDF with validation

## Error Handling

All operations return VNPT-CA compatible error codes (0-11):

| Code | Error | Cause |
|------|-------|-------|
| 0 | Success | Operation completed |
| 1 | InvalidInput | Empty/invalid file |
| 2 | CertificateNotFound | No cert on token |
| 3 | SigningFailed | Signature operation failed |
| 8 | TokenNotFound | No token detected |
| 11 | UserCancelled | User action required |

## Documentation

| Document | Purpose |
|----------|---------|
| [project-overview-pdr.md](./docs/project-overview-pdr.md) | Requirements, success criteria, risk assessment |
| [system-architecture.md](./docs/system-architecture.md) | Technical design, module responsibilities, data flow |
| [codebase-summary.md](./docs/codebase-summary.md) | Code organization, dependencies |
| [code-standards.md](./docs/code-standards.md) | Naming conventions, patterns, security practices |
| [project-roadmap.md](./docs/project-roadmap.md) | Release roadmap and future plans |

## Release Status

| Milestone | Status | Completion |
|-----------|--------|------------|
| Phase 1: Foundation | ✅ Complete | 2025-12-26 |
| Phase 2: Token Integration | ✅ Complete | 2025-12-26 |
| Phase 3: PDF Signing | ✅ Complete | 2025-12-26 |
| Phase 4: UI Implementation | ✅ Complete | 2025-12-26 |
| Phase 5: Testing & Distribution | ✅ Complete | 2025-12-27 |
| **v0.1.0 Release** | ✅ Released | 2025-12-27 |

**Total LOC:** ~4,607 (Frontend: 1,290, Backend: 3,317)
**Tests:** 96 total tests (unit + integration)
**Quality:** 0 TypeScript errors, 0 Rust warnings, 0 security vulnerabilities

## Dependencies

### Frontend
- React 18.3.1
- TypeScript 5.7.2
- Vite 6.0.3
- Tailwind CSS 3.4.16
- @tauri-apps/api 2.1.1

### Backend
- Tauri 2.x
- cryptoki 0.7 (PKCS#11)
- lopdf 0.34 (PDF manipulation)
- sha2 0.10 (SHA-256)
- x509-parser 0.16 (X.509 certificates)
- reqwest 0.12 (HTTP client)
- tokio 1.x (Async runtime)
- zeroize 1.x (Secure memory clearing)

**Vulnerability Status:** 0 known issues (npm audit + cargo audit)

## Security Considerations

### PIN Protection
- Never stored or logged
- Entered via secure dialog only
- Zeroized immediately after use
- 3-retry limit before token lock

### PKCS#11 Library Validation
- Hardcoded paths for known CAs
- No arbitrary library loading
- Path traversal prevention

### PDF Signing
- No JavaScript access to private keys
- Signatures created only in Rust backend
- Hash computation isolated
- Certificate chain embedded in signature

### Communication
- Tauri IPC enforces command validation
- TSA requests via HTTPS TLS 1.2+
- Content Security Policy enforced

## Building for Distribution

### macOS
```bash
npm run tauri build -- --target universal-apple-darwin
# Requires code signing certificate and notarization setup
```

### Windows
```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
# Requires Authenticode certificate for signing
```

## Troubleshooting

### Token Not Detected
- Check CA library is installed in standard path
- Verify token is inserted and powered
- Try unplugging and reinserting token

### PDF Signing Fails
- Ensure PDF is not corrupted or password-protected
- Verify certificate hasn't expired
- Check disk space for output file

### PIN Authentication Error
- Verify correct PIN entry (case-sensitive)
- Check remaining PIN attempts (3 max before lock)
- Request PIN reset from CA if locked

## Performance Metrics

| Operation | Target | Status |
|-----------|--------|--------|
| App Startup | <3 seconds | ✓ Achieved |
| Token Detection | <2 seconds | ✓ Achieved |
| Certificate Retrieval | <1 second | ✓ Achieved |
| PDF Signing | <2 seconds | ✓ Target |
| Memory Usage (idle) | <150 MB | ✓ Target |
| Binary Size | <25 MB | ✓ Achieved (18-20 MB) |

## Contributing

1. Read [docs/code-standards.md](./docs/code-standards.md) for coding guidelines
2. Follow naming conventions (camelCase TS, snake_case Rust, PascalCase types)
3. Ensure `npm run build` and `cargo clippy` pass
4. Add tests for new features (Phase 5)
5. Update relevant documentation

## License

MIT License - Copyright © 2025 Konek

## Support & Contact

- **Issues:** https://github.com/vieterp/esign.konek.vn/issues
- **Documentation:** See `/docs` directory
- **Repository:** https://github.com/vieterp/esign.konek.vn

## Future Roadmap

### v0.2.0 (Planned)
- Apple code signing (macOS notarization)
- Windows Authenticode signing
- Auto-update mechanism
- Enhanced logging infrastructure
- Extended test coverage

### v1.0.0+ (Planned)
- Mobile application (Flutter)
- Backend services (Fastify)
- Enterprise deployment options
- Batch signing support
- Advanced certificate management

## Project Statistics

- **Total LOC:** ~4,607 (Frontend: 1,290, Backend: 3,317)
- **Module Count:** 6 Rust modules + 4 React components + 2 hooks
- **Documentation:** Comprehensive across all phases
- **Dependencies:** 14 Rust crates + 8 npm packages
- **Build Time:** <3 minutes (macOS)
- **Binary Size:** 18-25 MB
- **Test Coverage:** 96 tests across unit and integration

## Getting Started for Developers

1. **Prerequisites:** Node.js 18+, Rust 1.70+
2. **Setup:** `npm install && npm run tauri dev`
3. **Code Quality:** `npx tsc --noEmit && cargo clippy`
4. **Testing:** Run complete test suite before PRs
5. **Documentation:** Keep docs in sync with code changes

---

**Last Updated:** 2025-12-27
**Current Version:** v0.1.0 (Released)
**Status:** Stable release with all core features complete

For detailed information, see [docs/project-overview-pdr.md](./docs/project-overview-pdr.md).
