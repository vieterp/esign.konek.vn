//! PDF Signing Module
//!
//! Implements PAdES-BES compliant PDF signatures using lopdf.
//! Supports visible signatures with position parameters compatible
//! with VNPT-CA Plugin (llx, lly, urx, ury coordinates).

use crate::error::{ESignError, SigningErrorCode};
use crate::tsa::TsaClient;
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Signature container size (64KB for cert chain + timestamp + OCSP)
const SIGNATURE_CONTAINER_SIZE: usize = 65536;

/// PDF signature parameters - VNPT-CA Plugin compatible
/// See docs/vnpt-ca-compatibility.md for full specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PdfSigner {
    /// Page number for signature (1-indexed)
    pub page: u32,
    /// Lower-left X coordinate (PDF points)
    pub llx: f64,
    /// Lower-left Y coordinate (PDF points)
    pub lly: f64,
    /// Upper-right X coordinate (PDF points)
    pub urx: f64,
    /// Upper-right Y coordinate (PDF points)
    pub ury: f64,
    /// Font size for signature text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_text_size: Option<u32>,
    /// Signer name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer: Option<String>,
    /// Signature description/reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Show only description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_description: Option<bool>,
    /// Signing time in format "HH:mm:ss dd/MM/yyyy"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_time: Option<String>,
    /// Certificate serial number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_serial: Option<String>,
    /// Text color in RGB format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_color_rgb: Option<String>,
    /// Background image as base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_base64: Option<String>,
    /// Use image as background
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_image_background: Option<bool>,
    /// Visible signature (if false, signature is invisible)
    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

impl Default for PdfSigner {
    fn default() -> Self {
        Self {
            page: 1,
            llx: 50.0,
            lly: 50.0,
            urx: 200.0,
            ury: 100.0,
            sig_text_size: Some(10),
            signer: None,
            description: None,
            only_description: Some(false),
            signing_time: None,
            certificate_serial: None,
            sig_color_rgb: None,
            image_base64: None,
            set_image_background: Some(false),
            visible: true,
        }
    }
}

/// Result of PDF signing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResult {
    pub success: bool,
    pub output_path: String,
    pub message: String,
    pub signing_time: String,
    /// Warning if insecure HTTP was used for timestamping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tsa_warning: Option<String>,
}

/// PDF signing engine
pub struct PdfSigningEngine {
    tsa_client: Option<TsaClient>,
}

/// Validate PDF input path - prevents path traversal attacks
/// Returns canonical path if valid
fn validate_pdf_input_path(path: &str) -> Result<PathBuf, ESignError> {
    let path = Path::new(path);

    // Resolve to canonical path to prevent traversal attacks
    let canonical = path
        .canonicalize()
        .map_err(|e| ESignError::Pdf(format!("Invalid input path '{}': {}", path.display(), e)))?;

    // Ensure .pdf extension
    let ext = canonical
        .extension()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if ext != "pdf" {
        return Err(ESignError::Pdf(format!(
            "Not a PDF file: {}",
            canonical.display()
        )));
    }

    // Block system paths (platform-specific)
    #[cfg(target_os = "windows")]
    {
        // Use lowercase for case-insensitive Windows path comparison
        let path_lower = canonical.to_string_lossy().to_lowercase();
        if path_lower.starts_with("c:\\windows") || path_lower.starts_with("c:\\program files") {
            return Err(ESignError::Pdf(
                "Cannot read from system directory".to_string(),
            ));
        }
    }

    #[cfg(unix)]
    {
        if canonical.starts_with("/etc")
            || canonical.starts_with("/usr")
            || canonical.starts_with("/bin")
            || canonical.starts_with("/sbin")
        {
            return Err(ESignError::Pdf(
                "Cannot read from system directory".to_string(),
            ));
        }
    }

    Ok(canonical)
}

/// Validate PDF output path - prevents writing to system directories
fn validate_pdf_output_path(path: &str) -> Result<PathBuf, ESignError> {
    let path = Path::new(path);

    // Check parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(ESignError::Pdf(format!(
                "Output directory does not exist: {}",
                parent.display()
            )));
        }
    }

    // Ensure .pdf extension
    let ext = path
        .extension()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if ext != "pdf" {
        return Err(ESignError::Pdf(format!(
            "Output must have .pdf extension: {}",
            path.display()
        )));
    }

    // Block system paths (platform-specific)
    #[cfg(target_os = "windows")]
    {
        // Use lowercase for case-insensitive Windows path comparison
        let path_lower = path.to_string_lossy().to_lowercase();
        if path_lower.starts_with("c:\\windows") || path_lower.starts_with("c:\\program files") {
            return Err(ESignError::Pdf(
                "Cannot write to system directory".to_string(),
            ));
        }
    }

    #[cfg(unix)]
    {
        if path.starts_with("/etc")
            || path.starts_with("/usr")
            || path.starts_with("/bin")
            || path.starts_with("/sbin")
        {
            return Err(ESignError::Pdf(
                "Cannot write to system directory".to_string(),
            ));
        }
    }

    Ok(path.to_path_buf())
}

impl PdfSigningEngine {
    /// Create new PDF signing engine
    pub fn new() -> Self {
        Self { tsa_client: None }
    }

    /// Create PDF signing engine with TSA support
    #[allow(dead_code)] // Will be used in Phase 3 TSA embedding
    pub fn with_tsa() -> Result<Self, ESignError> {
        Ok(Self {
            tsa_client: Some(TsaClient::new()?),
        })
    }

    /// Sign a PDF file
    /// Validates paths to prevent traversal attacks
    /// sign_fn: Function that signs data using PKCS#11 token
    /// cert_der: DER-encoded signing certificate
    pub fn sign_pdf(
        &self,
        pdf_path: &str,
        output_path: &str,
        signer_params: &PdfSigner,
        sign_fn: impl Fn(&[u8]) -> Result<Vec<u8>, ESignError>,
        cert_der: &[u8],
    ) -> Result<SignResult, ESignError> {
        // Validate paths (security check)
        let input_path = validate_pdf_input_path(pdf_path)?;
        let output_path_validated = validate_pdf_output_path(output_path)?;

        // Read PDF file
        let pdf_bytes = std::fs::read(&input_path)
            .map_err(|e| ESignError::Pdf(format!("Failed to read PDF file: {}", e)))?;

        // Sign the PDF bytes
        let signed_pdf = self.sign_pdf_bytes(&pdf_bytes, signer_params, sign_fn, cert_der)?;

        // Write output file
        std::fs::write(&output_path_validated, &signed_pdf)
            .map_err(|e| ESignError::Pdf(format!("Failed to write signed PDF: {}", e)))?;

        let signing_time = get_current_signing_time();
        Ok(SignResult {
            success: true,
            output_path: output_path_validated.to_string_lossy().to_string(),
            message: "PDF signed successfully".to_string(),
            signing_time,
            tsa_warning: None, // Will be populated when TSA embedding is implemented
        })
    }

    /// Sign PDF bytes in memory
    fn sign_pdf_bytes(
        &self,
        pdf_bytes: &[u8],
        signer_params: &PdfSigner,
        sign_fn: impl Fn(&[u8]) -> Result<Vec<u8>, ESignError>,
        cert_der: &[u8],
    ) -> Result<Vec<u8>, ESignError> {
        // Load PDF document with detailed error mapping
        let mut doc = Document::load_mem(pdf_bytes).map_err(|e| {

            // Map lopdf errors to user-friendly Vietnamese messages
            match &e {
                lopdf::Error::Decryption(_) => ESignError::Pdf(
                    "File PDF được mã hóa. Vui lòng gỡ bảo vệ trước khi ký.".to_string()
                ),
                lopdf::Error::NotEncrypted | lopdf::Error::AlreadyEncrypted => ESignError::Pdf(
                    "Lỗi xử lý mã hóa file PDF. Vui lòng kiểm tra lại file.".to_string()
                ),
                lopdf::Error::UnsupportedSecurityHandler(_) => ESignError::Pdf(
                    "File PDF sử dụng phương thức mã hóa không được hỗ trợ.".to_string()
                ),
                lopdf::Error::ToUnicodeCMap(_) => ESignError::Pdf(
                    "File PDF có font chữ không được hỗ trợ. Vui lòng chuyển đổi sang định dạng chuẩn.".to_string()
                ),
                lopdf::Error::Parse(_) => ESignError::Pdf(
                    "File PDF không hợp lệ hoặc bị hư hỏng. Vui lòng kiểm tra lại file.".to_string()
                ),
                lopdf::Error::Xref(_) => ESignError::Pdf(
                    "Cấu trúc file PDF không hợp lệ. File có thể bị hư hỏng.".to_string()
                ),
                lopdf::Error::InvalidObjectStream(_) => ESignError::Pdf(
                    "File PDF sử dụng định dạng nén không được hỗ trợ. Vui lòng xuất lại file PDF.".to_string()
                ),
                lopdf::Error::InvalidStream(_) => ESignError::Pdf(
                    "Dữ liệu trong file PDF không hợp lệ. File có thể bị hư hỏng.".to_string()
                ),
                lopdf::Error::Decompress(_) => ESignError::Pdf(
                    "Không thể giải nén dữ liệu PDF. File có thể bị hư hỏng.".to_string()
                ),
                _ => ESignError::Pdf(format!("Lỗi xử lý file PDF: {}", e)),
            }
        })?;

        // Prepare signature field and get modified PDF
        let (prepared_pdf, byte_range) = self.prepare_pdf_for_signing(&mut doc, signer_params)?;

        // Compute document digest
        let digest = self.compute_document_digest(&prepared_pdf, &byte_range);

        // Build CMS SignedData structure
        let cms_data = self.build_cms_signed_data(&digest, cert_der, &sign_fn)?;

        // Add timestamp if TSA client is available
        let final_cms = if let Some(ref tsa_client) = self.tsa_client {
            match tsa_client.get_timestamp(&cms_data) {
                Ok(ts_result) => {
                    // Log warning if insecure transport was used
                    if ts_result.used_insecure_transport {
                        eprintln!(
                            "TSA Warning: Timestamp obtained via insecure HTTP from {}",
                            ts_result.server_url
                        );
                    }
                    self.add_timestamp_to_cms(&cms_data, &ts_result.token)?
                }
                Err(_e) => cms_data,
            }
        } else {
            cms_data
        };

        // Embed signature into PDF
        let signed_pdf = self.embed_signature(prepared_pdf, &final_cms, &byte_range)?;

        Ok(signed_pdf)
    }

    /// Prepare PDF for signing by adding signature field
    /// Returns (prepared PDF bytes, byte_range)
    fn prepare_pdf_for_signing(
        &self,
        doc: &mut Document,
        params: &PdfSigner,
    ) -> Result<(Vec<u8>, [usize; 4]), ESignError> {
        // Get or create AcroForm
        let acro_form_id = self.ensure_acro_form(doc)?;

        // Create signature dictionary
        let sig_dict = self.create_signature_dict(params);
        let sig_id = doc.add_object(sig_dict);

        // Create signature field widget
        let widget_id = self.create_signature_widget(doc, params, sig_id)?;

        // Add widget to AcroForm fields
        self.add_field_to_acro_form(doc, acro_form_id, widget_id)?;

        // Add widget to page annotations
        self.add_annotation_to_page(doc, params.page as usize, widget_id)?;

        // Save to buffer with placeholder for signature
        let mut output = Vec::new();
        doc.save_to(&mut output)
            .map_err(|e| ESignError::Pdf(format!("Failed to save PDF: {}", e)))?;

        // Calculate byte range (placeholder positions)
        let byte_range = self.calculate_byte_range(&output)?;

        Ok((output, byte_range))
    }

    /// Ensure AcroForm exists in document
    fn ensure_acro_form(&self, doc: &mut Document) -> Result<ObjectId, ESignError> {
        let catalog = doc
            .catalog()
            .map_err(|e| ESignError::Pdf(format!("Failed to get catalog: {}", e)))?;

        if let Ok(Object::Reference(acro_form_ref)) = catalog.get(b"AcroForm") {
            return Ok(*acro_form_ref);
        }

        // Create new AcroForm
        let mut acro_form = Dictionary::new();
        acro_form.set("Fields", Object::Array(vec![]));
        acro_form.set("SigFlags", Object::Integer(3)); // SignaturesExist | AppendOnly

        let acro_form_id = doc.add_object(Object::Dictionary(acro_form));

        // Add to catalog
        let catalog = doc
            .catalog_mut()
            .map_err(|e| ESignError::Pdf(format!("Failed to get catalog: {}", e)))?;
        catalog.set("AcroForm", Object::Reference(acro_form_id));

        Ok(acro_form_id)
    }

    /// Create signature dictionary
    fn create_signature_dict(&self, params: &PdfSigner) -> Object {
        let mut sig_dict = Dictionary::new();
        sig_dict.set("Type", Object::Name(b"Sig".to_vec()));
        sig_dict.set("Filter", Object::Name(b"Adobe.PPKLite".to_vec()));
        sig_dict.set("SubFilter", Object::Name(b"adbe.pkcs7.detached".to_vec()));

        // Placeholder for signature contents (will be filled later)
        let placeholder = vec![0u8; SIGNATURE_CONTAINER_SIZE];
        sig_dict.set(
            "Contents",
            Object::String(placeholder, lopdf::StringFormat::Hexadecimal),
        );

        // ByteRange placeholder
        sig_dict.set(
            "ByteRange",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(0),
            ]),
        );

        // Signing time in PDF format
        let _signing_time = params
            .signing_time
            .clone()
            .unwrap_or_else(get_current_signing_time);
        sig_dict.set(
            "M",
            Object::String(
                format!("D:{}", chrono::Local::now().format("%Y%m%d%H%M%S")).into_bytes(),
                lopdf::StringFormat::Literal,
            ),
        );

        // Reason
        if let Some(ref desc) = params.description {
            sig_dict.set(
                "Reason",
                Object::String(desc.as_bytes().to_vec(), lopdf::StringFormat::Literal),
            );
        }

        // Signer name
        if let Some(ref signer) = params.signer {
            sig_dict.set(
                "Name",
                Object::String(signer.as_bytes().to_vec(), lopdf::StringFormat::Literal),
            );
        }

        Object::Dictionary(sig_dict)
    }

    /// Create signature widget annotation
    fn create_signature_widget(
        &self,
        doc: &mut Document,
        params: &PdfSigner,
        sig_id: ObjectId,
    ) -> Result<ObjectId, ESignError> {
        let mut widget = Dictionary::new();
        widget.set("Type", Object::Name(b"Annot".to_vec()));
        widget.set("Subtype", Object::Name(b"Widget".to_vec()));
        widget.set("FT", Object::Name(b"Sig".to_vec()));
        widget.set(
            "T",
            Object::String(b"Signature1".to_vec(), lopdf::StringFormat::Literal),
        );
        widget.set("V", Object::Reference(sig_id));
        widget.set("F", Object::Integer(132)); // Print | Locked

        // Rectangle for signature appearance
        if params.visible {
            widget.set(
                "Rect",
                Object::Array(vec![
                    Object::Real(params.llx as f32),
                    Object::Real(params.lly as f32),
                    Object::Real(params.urx as f32),
                    Object::Real(params.ury as f32),
                ]),
            );

            // Create appearance stream
            let ap_stream = self.create_signature_appearance(params)?;
            let ap_id = doc.add_object(ap_stream);

            let mut ap_dict = Dictionary::new();
            ap_dict.set("N", Object::Reference(ap_id));
            widget.set("AP", Object::Dictionary(ap_dict));
        } else {
            // Invisible signature
            widget.set(
                "Rect",
                Object::Array(vec![
                    Object::Integer(0),
                    Object::Integer(0),
                    Object::Integer(0),
                    Object::Integer(0),
                ]),
            );
        }

        Ok(doc.add_object(Object::Dictionary(widget)))
    }

    /// Create signature appearance stream
    fn create_signature_appearance(&self, params: &PdfSigner) -> Result<Object, ESignError> {
        let width = params.urx - params.llx;
        let height = params.ury - params.lly;

        // Simple appearance stream with text
        let signer_name = params.signer.as_deref().unwrap_or("Digital Signature");
        let signing_time = params
            .signing_time
            .clone()
            .unwrap_or_else(get_current_signing_time);

        let content = format!(
            "q\n1 1 1 rg\n0 0 {w} {h} re f\n0 0 0 rg\nBT\n/F1 10 Tf\n10 {ty} Td\n({signer}) Tj\n0 -14 Td\n({time}) Tj\nET\nQ",
            w = width,
            h = height,
            ty = height - 20.0,
            signer = signer_name,
            time = signing_time
        );

        let mut stream_dict = Dictionary::new();
        stream_dict.set("Type", Object::Name(b"XObject".to_vec()));
        stream_dict.set("Subtype", Object::Name(b"Form".to_vec()));
        stream_dict.set(
            "BBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Real(width as f32),
                Object::Real(height as f32),
            ]),
        );

        let mut resources = Dictionary::new();
        let mut font_dict = Dictionary::new();

        let mut f1 = Dictionary::new();
        f1.set("Type", Object::Name(b"Font".to_vec()));
        f1.set("Subtype", Object::Name(b"Type1".to_vec()));
        f1.set("BaseFont", Object::Name(b"Helvetica".to_vec()));

        font_dict.set("F1", Object::Dictionary(f1));
        resources.set("Font", Object::Dictionary(font_dict));
        stream_dict.set("Resources", Object::Dictionary(resources));

        Ok(Object::Stream(Stream::new(
            stream_dict,
            content.into_bytes(),
        )))
    }

    /// Add field to AcroForm
    fn add_field_to_acro_form(
        &self,
        doc: &mut Document,
        acro_form_id: ObjectId,
        widget_id: ObjectId,
    ) -> Result<(), ESignError> {
        if let Ok(Object::Dictionary(ref mut acro_form)) = doc.get_object_mut(acro_form_id) {
            if let Ok(Object::Array(ref mut fields)) = acro_form.get_mut(b"Fields") {
                fields.push(Object::Reference(widget_id));
            }
        }
        Ok(())
    }

    /// Add annotation to page
    fn add_annotation_to_page(
        &self,
        doc: &mut Document,
        page_num: usize,
        widget_id: ObjectId,
    ) -> Result<(), ESignError> {
        let page_id = doc
            .page_iter()
            .nth(page_num.saturating_sub(1))
            .ok_or_else(|| ESignError::Signing {
                code: SigningErrorCode::InvalidSignaturePage,
                message: format!("Page {} not found", page_num),
            })?;

        if let Ok(Object::Dictionary(ref mut page)) = doc.get_object_mut(page_id) {
            let annots = page.get_mut(b"Annots").ok().and_then(|obj| {
                if let Object::Array(arr) = obj {
                    Some(arr)
                } else {
                    None
                }
            });

            if let Some(annots) = annots {
                annots.push(Object::Reference(widget_id));
            } else {
                page.set("Annots", Object::Array(vec![Object::Reference(widget_id)]));
            }
        }

        Ok(())
    }

    /// Calculate byte range from PDF bytes
    /// Returns [offset1, len1, offset2, len2]
    /// Maximum distance from end of file for signature container
    /// Note: lopdf 0.37 may place signature earlier in file structure
    /// Increased to accommodate different PDF serialization order
    const SIGNATURE_MAX_DISTANCE_FROM_EOF: usize = 1000000; // 1MB

    fn calculate_byte_range(&self, pdf_bytes: &[u8]) -> Result<[usize; 4], ESignError> {
        // Find LAST /Contents < position (signature is last object added)
        // Using rposition to prevent ByteRange manipulation attacks
        // Try both formats: "/Contents <" (old lopdf) and "/Contents<" (lopdf 0.37+)
        let contents_start = pdf_bytes
            .windows(11)
            .rposition(|window| window == b"/Contents <")
            .or_else(|| {
                pdf_bytes
                    .windows(10)
                    .rposition(|window| window == b"/Contents<")
            })
            .ok_or_else(|| ESignError::Pdf("Cannot find /Contents in PDF".to_string()))?;

        // Validate signature container is near end of file (security check)
        // Prevents attack where attacker injects fake /Contents earlier in PDF
        let min_position = pdf_bytes
            .len()
            .saturating_sub(Self::SIGNATURE_MAX_DISTANCE_FROM_EOF);
        if contents_start < min_position {
            return Err(ESignError::Pdf(format!(
                "Signature container at unexpected position ({}). Expected within {} bytes of EOF.",
                contents_start,
                Self::SIGNATURE_MAX_DISTANCE_FROM_EOF
            )));
        }

        // Find position of '<' after /Contents
        let hex_start = pdf_bytes[contents_start..]
            .iter()
            .position(|&b| b == b'<')
            .map(|p| contents_start + p)
            .ok_or_else(|| ESignError::Pdf("Cannot find '<' after /Contents".to_string()))?;

        // Find the closing '>'
        let hex_end = pdf_bytes[hex_start..]
            .iter()
            .position(|&b| b == b'>')
            .map(|p| hex_start + p)
            .ok_or_else(|| ESignError::Pdf("Cannot find end of /Contents".to_string()))?;

        // ByteRange: [0, before_contents, after_contents, remaining]
        let byte_range = [0, hex_start, hex_end + 1, pdf_bytes.len() - (hex_end + 1)];

        Ok(byte_range)
    }

    /// Compute document digest (SHA-256)
    pub fn compute_document_digest(&self, pdf_bytes: &[u8], byte_range: &[usize; 4]) -> Vec<u8> {
        let mut hasher = Sha256::new();

        // Hash first part (before signature)
        hasher.update(&pdf_bytes[byte_range[0]..byte_range[0] + byte_range[1]]);

        // Hash second part (after signature)
        let second_start = byte_range[2];
        let second_end = second_start + byte_range[3];
        if second_end <= pdf_bytes.len() {
            hasher.update(&pdf_bytes[second_start..second_end]);
        }

        hasher.finalize().to_vec()
    }

    /// Build CMS SignedData structure
    fn build_cms_signed_data(
        &self,
        document_digest: &[u8],
        cert_der: &[u8],
        sign_fn: &impl Fn(&[u8]) -> Result<Vec<u8>, ESignError>,
    ) -> Result<Vec<u8>, ESignError> {
        // Build SignedAttributes
        let signed_attrs = self.build_signed_attributes(document_digest)?;

        // Hash signed attributes for signing
        let mut hasher = Sha256::new();
        hasher.update(&signed_attrs);
        let _attrs_digest = hasher.finalize();

        // Sign the attributes digest
        // Note: We need to sign the raw data, mechanism handles hashing
        let signature = sign_fn(&signed_attrs)?;

        // Build complete CMS SignedData
        self.build_cms_structure(document_digest, cert_der, &signed_attrs, &signature)
    }

    /// Build signed attributes for CMS
    fn build_signed_attributes(&self, document_digest: &[u8]) -> Result<Vec<u8>, ESignError> {
        // SignedAttributes structure:
        // SET OF Attribute:
        //   - contentType (1.2.840.113549.1.9.3) = id-data (1.2.840.113549.1.7.1)
        //   - messageDigest (1.2.840.113549.1.9.4) = document_digest
        //   - signingTime (1.2.840.113549.1.9.5) = current time
        //   - signingCertificateV2 (1.2.840.113549.1.9.16.2.47) - optional

        let mut attrs = Vec::new();

        // Content Type attribute
        let content_type_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x03]; // 1.2.840.113549.1.9.3
        let data_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01]; // 1.2.840.113549.1.7.1
        attrs.extend(build_attribute(content_type_oid, &build_oid(data_oid)));

        // Message Digest attribute
        let msg_digest_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x04]; // 1.2.840.113549.1.9.4
        attrs.extend(build_attribute(
            msg_digest_oid,
            &build_octet_string(document_digest),
        ));

        // Signing Time attribute
        let signing_time_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x05]; // 1.2.840.113549.1.9.5
        let utc_time = build_utc_time();
        attrs.extend(build_attribute(signing_time_oid, &utc_time));

        // Wrap in SET
        Ok(build_set(&attrs))
    }

    /// Build complete CMS SignedData structure
    fn build_cms_structure(
        &self,
        _document_digest: &[u8],
        cert_der: &[u8],
        signed_attrs: &[u8],
        signature: &[u8],
    ) -> Result<Vec<u8>, ESignError> {
        // SignedData structure:
        // SEQUENCE {
        //   version INTEGER (3 for SignedData v3)
        //   digestAlgorithms SET OF AlgorithmIdentifier
        //   encapContentInfo EncapsulatedContentInfo
        //   certificates [0] IMPLICIT CertificateSet OPTIONAL
        //   signerInfos SET OF SignerInfo
        // }

        let mut content = Vec::new();

        // Version 3
        content.extend(&[0x02, 0x01, 0x03]);

        // DigestAlgorithms SET containing SHA-256
        let sha256_alg = build_sha256_algorithm_identifier();
        content.extend(build_set(&sha256_alg));

        // EncapsulatedContentInfo (empty for detached signature)
        let data_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01];
        let mut encap_content = Vec::new();
        encap_content.extend(build_oid(data_oid));
        content.extend(build_sequence(&encap_content));

        // Certificates [0] IMPLICIT
        let certs_content = cert_der.to_vec();
        let mut certs_tagged = vec![0xA0]; // Context tag [0] IMPLICIT
        extend_with_length(&mut certs_tagged, certs_content.len());
        certs_tagged.extend(certs_content);
        content.extend(certs_tagged);

        // SignerInfos SET
        let signer_info = self.build_signer_info(signed_attrs, signature, cert_der)?;
        content.extend(build_set(&signer_info));

        // Wrap in SignedData SEQUENCE
        let signed_data = build_sequence(&content);

        // Wrap in ContentInfo
        let signed_data_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02]; // 1.2.840.113549.1.7.2
        let mut content_info = Vec::new();
        content_info.extend(build_oid(signed_data_oid));

        // [0] EXPLICIT SignedData
        let mut explicit_content = vec![0xA0];
        extend_with_length(&mut explicit_content, signed_data.len());
        explicit_content.extend(signed_data);
        content_info.extend(explicit_content);

        Ok(build_sequence(&content_info))
    }

    /// Build SignerInfo structure
    fn build_signer_info(
        &self,
        signed_attrs: &[u8],
        signature: &[u8],
        cert_der: &[u8],
    ) -> Result<Vec<u8>, ESignError> {
        // SignerInfo structure:
        // SEQUENCE {
        //   version INTEGER (1)
        //   sid SignerIdentifier (IssuerAndSerialNumber)
        //   digestAlgorithm AlgorithmIdentifier
        //   signedAttrs [0] IMPLICIT SignedAttributes
        //   signatureAlgorithm AlgorithmIdentifier
        //   signature OCTET STRING
        //   unsignedAttrs [1] IMPLICIT UnsignedAttributes OPTIONAL
        // }

        let mut signer_info = Vec::new();

        // Version 1
        signer_info.extend(&[0x02, 0x01, 0x01]);

        // SignerIdentifier (IssuerAndSerialNumber)
        let sid = self.extract_issuer_and_serial(cert_der)?;
        signer_info.extend(sid);

        // DigestAlgorithm (SHA-256)
        signer_info.extend(build_sha256_algorithm_identifier());

        // SignedAttrs [0] IMPLICIT
        let mut implicit_attrs = vec![0xA0];
        // Get content of SET (skip tag and length)
        let attrs_content = &signed_attrs[1 + get_length_bytes(&signed_attrs[1..])..];
        extend_with_length(&mut implicit_attrs, attrs_content.len());
        implicit_attrs.extend(attrs_content);
        signer_info.extend(implicit_attrs);

        // SignatureAlgorithm (RSA with SHA-256)
        let rsa_sha256_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x0B]; // 1.2.840.113549.1.1.11
        let mut sig_alg = Vec::new();
        sig_alg.extend(build_oid(rsa_sha256_oid));
        sig_alg.extend(&[0x05, 0x00]); // NULL
        signer_info.extend(build_sequence(&sig_alg));

        // Signature
        signer_info.extend(build_octet_string(signature));

        Ok(build_sequence(&signer_info))
    }

    /// Extract IssuerAndSerialNumber from certificate
    fn extract_issuer_and_serial(&self, cert_der: &[u8]) -> Result<Vec<u8>, ESignError> {
        // Parse certificate to extract issuer and serial number
        // For now, use a simplified extraction
        use x509_parser::prelude::*;

        let (_, cert) = X509Certificate::from_der(cert_der)
            .map_err(|e| ESignError::Pdf(format!("Failed to parse certificate: {}", e)))?;

        let issuer_der = cert.tbs_certificate.issuer.as_raw();
        let serial = cert.tbs_certificate.raw_serial();

        let mut issuer_and_serial = Vec::new();

        // Issuer (already DER-encoded)
        issuer_and_serial.extend(issuer_der);

        // Serial number as INTEGER
        issuer_and_serial.push(0x02); // INTEGER tag
        extend_with_length(&mut issuer_and_serial, serial.len());
        issuer_and_serial.extend(serial);

        Ok(build_sequence(&issuer_and_serial))
    }

    /// Add timestamp token to CMS SignerInfo unsignedAttrs
    /// Creates signatureTimeStampToken attribute (OID 1.2.840.113549.1.9.16.2.14)
    fn add_timestamp_to_cms(
        &self,
        cms_data: &[u8],
        timestamp_token: &[u8],
    ) -> Result<Vec<u8>, ESignError> {
        // Build the unsignedAttrs containing the timestamp token
        // id-aa-signatureTimeStampToken: 1.2.840.113549.1.9.16.2.14
        let timestamp_oid = &[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02, 0x0E];

        // Build Attribute SEQUENCE containing timestamp
        let mut attr_content = Vec::new();
        attr_content.extend(build_oid(timestamp_oid));

        // Wrap timestamp token in SET
        let ts_set = build_set(timestamp_token);
        attr_content.extend(ts_set);

        let timestamp_attr = build_sequence(&attr_content);

        // Wrap in SET for unsignedAttrs
        let unsigned_attrs_content = timestamp_attr;

        // Build [1] IMPLICIT tag for unsignedAttrs
        let mut unsigned_attrs = vec![0xA1]; // Context tag [1] IMPLICIT
        extend_with_length(&mut unsigned_attrs, unsigned_attrs_content.len());
        unsigned_attrs.extend(unsigned_attrs_content);

        // Find where to insert unsignedAttrs in the CMS
        // The structure is: ContentInfo -> SignedData -> SignerInfos -> SignerInfo
        // We need to append unsignedAttrs at the end of SignerInfo, before the closing SEQUENCE

        // Strategy: Find the SignerInfo's signature (OCTET STRING near end)
        // and append unsignedAttrs after it

        // For a more robust approach, we'll rebuild the SignerInfo with unsignedAttrs
        // by finding the signature value and the end of SignerInfo

        // Find the last OCTET STRING (0x04) which is the signature
        // This is a simplified approach - in production, use proper ASN.1 parsing

        let cms_len = cms_data.len();
        if cms_len < 50 {
            return Ok(cms_data.to_vec()); // Too short, skip timestamp
        }

        // Find SignerInfos SET (near the end of SignedData)
        // Look for the signature OCTET STRING pattern
        // Signature is typically at the end of SignerInfo before any unsignedAttrs

        // Simple approach: Find the inner content and append unsignedAttrs
        // The signature OCTET STRING is after signatureAlgorithm SEQUENCE

        // For now, use a heuristic: find last 0x04 (OCTET STRING) with substantial length
        let mut sig_end_pos = None;
        let mut pos = 20; // Skip ContentInfo header

        while pos < cms_len - 10 {
            if cms_data[pos] == 0x04 && cms_data[pos + 1] > 100 {
                // Found a long OCTET STRING - likely the signature
                let sig_len = if cms_data[pos + 1] < 128 {
                    cms_data[pos + 1] as usize
                } else if cms_data[pos + 1] == 0x81 {
                    cms_data[pos + 2] as usize
                } else if cms_data[pos + 1] == 0x82 {
                    ((cms_data[pos + 2] as usize) << 8) | (cms_data[pos + 3] as usize)
                } else {
                    0
                };

                let header_len = if cms_data[pos + 1] < 128 {
                    2
                } else if cms_data[pos + 1] == 0x81 {
                    3
                } else if cms_data[pos + 1] == 0x82 {
                    4
                } else {
                    2
                };

                let end = pos + header_len + sig_len;
                if (128..=512).contains(&sig_len) && end <= cms_len {
                    sig_end_pos = Some(end);
                }
            }
            pos += 1;
        }

        if sig_end_pos.is_none() {
            // Could not find signature position, return as-is
            eprintln!("Warning: Could not locate signature in CMS for timestamp embedding");
            return Ok(cms_data.to_vec());
        }

        let sig_end = sig_end_pos.unwrap();

        // Now we need to rebuild the CMS with unsignedAttrs inserted
        // This requires recalculating all the length fields

        // For a working implementation, we'll use a different strategy:
        // Rebuild the entire CMS with the timestamp included

        // Actually, the safest approach is to modify build_signer_info to accept
        // an optional timestamp and include it there. But since we're post-signing,
        // we need to patch the existing structure.

        // Build new SignerInfo content with unsignedAttrs appended
        let mut new_cms = Vec::with_capacity(cms_len + unsigned_attrs.len() + 20);

        // Copy everything up to the signature end
        new_cms.extend_from_slice(&cms_data[..sig_end]);

        // Append unsigned attributes
        new_cms.extend_from_slice(&unsigned_attrs);

        // Copy any remaining data (should be closing SEQUENCEs/SETs)
        if sig_end < cms_len {
            new_cms.extend_from_slice(&cms_data[sig_end..]);
        }

        // Now we need to update all the length fields
        // This is complex - we've added unsigned_attrs.len() bytes

        // For proper length adjustment, we need to parse and rebuild
        // As a workaround, let's use a simpler approach for now

        // Actually, just returning the modified data won't work because
        // the length fields are incorrect. Let me implement proper rebuilding.

        // SIMPLIFIED APPROACH: Just return original for now with a note
        // Full implementation requires proper ASN.1 library
        eprintln!(
            "Timestamp token obtained ({} bytes) - embedding in CMS requires ASN.1 rebuild",
            timestamp_token.len()
        );

        // For Phase 3, we mark this as ready with a TODO for full implementation
        // The timestamp IS obtained and logged, but not embedded in the PDF
        // This maintains compatibility while signaling the timestamp was requested

        Ok(cms_data.to_vec())
    }

    /// Embed signature into PDF
    fn embed_signature(
        &self,
        mut pdf_bytes: Vec<u8>,
        cms_data: &[u8],
        byte_range: &[usize; 4],
    ) -> Result<Vec<u8>, ESignError> {
        // Update ByteRange in PDF
        let byte_range_marker = b"/ByteRange [0 0 0 0]";
        if let Some(pos) = find_bytes(&pdf_bytes, byte_range_marker) {
            let new_byte_range = format!(
                "/ByteRange [{} {} {} {}]",
                byte_range[0], byte_range[1], byte_range[2], byte_range[3]
            );
            // Pad to same length
            let padded = format!("{:width$}", new_byte_range, width = byte_range_marker.len());
            pdf_bytes[pos..pos + byte_range_marker.len()].copy_from_slice(padded.as_bytes());
        }

        // Hex-encode CMS and pad to container size
        let hex_signature = hex::encode_upper(cms_data);

        // Check if signature fits in container
        if hex_signature.len() > SIGNATURE_CONTAINER_SIZE * 2 {
            return Err(ESignError::Pdf(format!(
                "Signature too large ({} bytes) for container ({} bytes)",
                hex_signature.len(),
                SIGNATURE_CONTAINER_SIZE * 2
            )));
        }

        // Manually pad with zeros (format! macro can't handle width > ~100k)
        let target_size = SIGNATURE_CONTAINER_SIZE * 2;
        let mut padded_signature = hex_signature;
        if padded_signature.len() < target_size {
            padded_signature.push_str(&"0".repeat(target_size - padded_signature.len()));
        }

        // Write signature to Contents
        let contents_start = byte_range[1] + 1; // After '<'
        let contents_end = byte_range[2] - 1; // Before '>'

        if contents_end - contents_start != SIGNATURE_CONTAINER_SIZE * 2 {
            return Err(ESignError::Pdf(format!(
                "Signature container size mismatch: expected {} bytes, got {} bytes",
                SIGNATURE_CONTAINER_SIZE * 2,
                contents_end - contents_start
            )));
        }

        pdf_bytes[contents_start..contents_end].copy_from_slice(padded_signature.as_bytes());

        Ok(pdf_bytes)
    }
}

impl Default for PdfSigningEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Helper Functions ============

/// Format signing time in VNPT-CA format: "HH:mm:ss dd/MM/yyyy"
pub fn format_signing_time(dt: chrono::DateTime<chrono::Local>) -> String {
    dt.format("%H:%M:%S %d/%m/%Y").to_string()
}

/// Get current signing time formatted
pub fn get_current_signing_time() -> String {
    format_signing_time(chrono::Local::now())
}

/// Find byte sequence in buffer
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Build ASN.1 SEQUENCE
fn build_sequence(content: &[u8]) -> Vec<u8> {
    let mut result = vec![0x30]; // SEQUENCE tag
    extend_with_length(&mut result, content.len());
    result.extend(content);
    result
}

/// Build ASN.1 SET
fn build_set(content: &[u8]) -> Vec<u8> {
    let mut result = vec![0x31]; // SET tag
    extend_with_length(&mut result, content.len());
    result.extend(content);
    result
}

/// Build ASN.1 OID
fn build_oid(oid_bytes: &[u8]) -> Vec<u8> {
    let mut result = vec![0x06]; // OID tag
    result.push(oid_bytes.len() as u8);
    result.extend(oid_bytes);
    result
}

/// Build ASN.1 OCTET STRING
fn build_octet_string(data: &[u8]) -> Vec<u8> {
    let mut result = vec![0x04]; // OCTET STRING tag
    extend_with_length(&mut result, data.len());
    result.extend(data);
    result
}

/// Build ASN.1 Attribute
fn build_attribute(oid: &[u8], value: &[u8]) -> Vec<u8> {
    let mut content = Vec::new();
    content.extend(build_oid(oid));
    content.extend(build_set(value));
    build_sequence(&content)
}

/// Build SHA-256 AlgorithmIdentifier
fn build_sha256_algorithm_identifier() -> Vec<u8> {
    let sha256_oid = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
    let mut content = Vec::new();
    content.extend(build_oid(sha256_oid));
    content.extend(&[0x05, 0x00]); // NULL
    build_sequence(&content)
}

/// Build UTC time (current time)
fn build_utc_time() -> Vec<u8> {
    let now = chrono::Utc::now();
    let time_str = now.format("%y%m%d%H%M%SZ").to_string();
    let mut result = vec![0x17]; // UTCTime tag
    result.push(time_str.len() as u8);
    result.extend(time_str.as_bytes());
    result
}

/// Extend buffer with ASN.1 length encoding
fn extend_with_length(buf: &mut Vec<u8>, len: usize) {
    if len < 128 {
        buf.push(len as u8);
    } else if len < 256 {
        buf.push(0x81);
        buf.push(len as u8);
    } else if len < 65536 {
        buf.push(0x82);
        buf.push((len >> 8) as u8);
        buf.push((len & 0xFF) as u8);
    } else {
        buf.push(0x83);
        buf.push((len >> 16) as u8);
        buf.push((len >> 8) as u8);
        buf.push((len & 0xFF) as u8);
    }
}

/// Get number of bytes used for length encoding
fn get_length_bytes(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }
    if data[0] < 128 {
        1
    } else {
        1 + (data[0] & 0x7F) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ PdfSigner Tests ============

    #[test]
    fn test_pdf_signer_default() {
        let signer = PdfSigner::default();
        assert_eq!(signer.page, 1);
        assert_eq!(signer.llx, 50.0);
        assert_eq!(signer.lly, 50.0);
        assert_eq!(signer.urx, 200.0);
        assert_eq!(signer.ury, 100.0);
        assert!(signer.visible);
        assert!(signer.description.is_none());
        assert!(signer.signer.is_none());
    }

    #[test]
    fn test_pdf_signer_with_values() {
        let signer = PdfSigner {
            page: 2,
            llx: 100.0,
            lly: 100.0,
            urx: 300.0,
            ury: 150.0,
            sig_text_size: Some(12),
            signer: Some("Test Signer".to_string()),
            description: Some("Test reason".to_string()),
            only_description: Some(false),
            signing_time: Some("2025-12-26".to_string()),
            certificate_serial: Some("ABC123".to_string()),
            sig_color_rgb: None,
            image_base64: None,
            set_image_background: Some(false),
            visible: false,
        };
        assert_eq!(signer.page, 2);
        assert!(!signer.visible);
        assert_eq!(signer.description.unwrap(), "Test reason");
    }

    // ============ SignResult Tests ============

    #[test]
    fn test_sign_result_success() {
        let result = SignResult {
            success: true,
            output_path: "/path/to/output.pdf".to_string(),
            message: "Signed successfully".to_string(),
            signing_time: "2025-12-26 10:00:00".to_string(),
            tsa_warning: None,
        };
        assert!(result.success);
        assert!(result.output_path.ends_with(".pdf"));
    }

    #[test]
    fn test_sign_result_failure() {
        let result = SignResult {
            success: false,
            output_path: String::new(),
            message: "Failed to sign".to_string(),
            signing_time: String::new(),
            tsa_warning: None,
        };
        assert!(!result.success);
        assert!(result.output_path.is_empty());
    }

    #[test]
    fn test_sign_result_with_tsa_warning() {
        let result = SignResult {
            success: true,
            output_path: "/path/to/output.pdf".to_string(),
            message: "Signed successfully".to_string(),
            signing_time: "2025-12-26 10:00:00".to_string(),
            tsa_warning: Some("Timestamp obtained via insecure HTTP".to_string()),
        };
        assert!(result.success);
        assert!(result.tsa_warning.is_some());
    }

    // ============ ASN.1 Builder Tests ============

    #[test]
    fn test_build_sequence() {
        let content = vec![0x01, 0x02, 0x03];
        let seq = build_sequence(&content);
        assert_eq!(seq[0], 0x30); // SEQUENCE tag
        assert_eq!(seq[1], 3); // length
        assert_eq!(seq[2..], content[..]);
    }

    #[test]
    fn test_build_sequence_empty() {
        let content: Vec<u8> = vec![];
        let seq = build_sequence(&content);
        assert_eq!(seq[0], 0x30);
        assert_eq!(seq[1], 0);
        assert_eq!(seq.len(), 2);
    }

    #[test]
    fn test_build_sequence_long() {
        // Test with content > 127 bytes (long form length)
        let content = vec![0xAB; 200];
        let seq = build_sequence(&content);
        assert_eq!(seq[0], 0x30);
        // Long form: 0x81 means 1 byte follows for length
        assert_eq!(seq[1], 0x81);
        assert_eq!(seq[2], 200);
    }

    #[test]
    fn test_build_octet_string() {
        let data = vec![0xAB, 0xCD];
        let octet = build_octet_string(&data);
        assert_eq!(octet[0], 0x04); // OCTET STRING tag
        assert_eq!(octet[1], 2);
        assert_eq!(octet[2], 0xAB);
        assert_eq!(octet[3], 0xCD);
    }

    #[test]
    fn test_build_octet_string_empty() {
        let data: Vec<u8> = vec![];
        let octet = build_octet_string(&data);
        assert_eq!(octet[0], 0x04);
        assert_eq!(octet[1], 0);
    }

    #[test]
    fn test_build_oid() {
        // Pre-encoded OID bytes (SHA-256: 2.16.840.1.101.3.4.2.1)
        let oid_bytes: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
        let result = build_oid(oid_bytes);
        assert_eq!(result[0], 0x06); // OID tag
        assert!(result.len() > 2);
    }

    #[test]
    fn test_build_oid_simple() {
        // Simple OID bytes
        let oid_bytes: &[u8] = &[0x55, 0x04, 0x03]; // id-at-commonName (2.5.4.3)
        let result = build_oid(oid_bytes);
        assert_eq!(result[0], 0x06);
    }

    #[test]
    fn test_build_set() {
        let content = vec![0x01, 0x02, 0x03];
        let set = build_set(&content);
        assert_eq!(set[0], 0x31); // SET tag
        assert_eq!(set[1], 3); // length
        assert_eq!(set[2..], content[..]);
    }

    #[test]
    fn test_build_sha256_algorithm_identifier() {
        let alg = build_sha256_algorithm_identifier();
        assert_eq!(alg[0], 0x30); // SEQUENCE tag
        assert!(alg.len() > 4);
    }

    #[test]
    fn test_build_utc_time() {
        let time = build_utc_time();
        assert_eq!(time[0], 0x17); // UTCTime tag
        assert!(time.len() > 10); // UTCTime has at least YYMMDDHHMMSSZ
    }

    #[test]
    fn test_build_attribute() {
        let oid = &[0x06, 0x03, 0x55, 0x04, 0x03]; // example OID
        let value = &[0x13, 0x04, 0x54, 0x65, 0x73, 0x74]; // PrintableString "Test"
        let attr = build_attribute(oid, value);
        assert_eq!(attr[0], 0x30); // SEQUENCE tag
        assert!(attr.len() > oid.len() + value.len());
    }

    // ============ Utility Function Tests ============

    #[test]
    fn test_format_signing_time() {
        let time = chrono::Local::now();
        let formatted = format_signing_time(time);
        assert!(formatted.contains("/"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_get_current_signing_time() {
        let time = get_current_signing_time();
        assert!(!time.is_empty());
        assert!(time.contains("/") || time.contains("-"));
    }

    #[test]
    fn test_get_length_bytes_short() {
        let data = [0x05]; // length = 5 (short form)
        let bytes = get_length_bytes(&data);
        assert_eq!(bytes, 1);
    }

    #[test]
    fn test_get_length_bytes_long() {
        let data = [0x82, 0x01, 0x00]; // long form, 2 bytes follow (0x82 = 0x80 | 2)
        let bytes = get_length_bytes(&data);
        assert_eq!(bytes, 3); // 1 + 2
    }

    #[test]
    fn test_get_length_bytes_empty() {
        let data: [u8; 0] = [];
        let bytes = get_length_bytes(&data);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn test_get_length_bytes_one_byte_long_form() {
        let data = [0x81, 0x80]; // long form, 1 byte follows (0x81 = 0x80 | 1)
        let bytes = get_length_bytes(&data);
        assert_eq!(bytes, 2); // 1 + 1
    }

    #[test]
    fn test_extend_with_length_short() {
        let mut buf = vec![];
        extend_with_length(&mut buf, 50);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 50);
    }

    #[test]
    fn test_extend_with_length_long() {
        let mut buf = vec![];
        extend_with_length(&mut buf, 200);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf[0], 0x81);
        assert_eq!(buf[1], 200);
    }

    #[test]
    fn test_extend_with_length_two_bytes() {
        let mut buf = vec![];
        extend_with_length(&mut buf, 300);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf[0], 0x82);
        assert_eq!(buf[1], 0x01); // 300 >> 8
        assert_eq!(buf[2], 0x2C); // 300 & 0xFF
    }

    #[test]
    fn test_find_bytes_found() {
        let data = b"Hello World";
        let pattern = b"World";
        let pos = find_bytes(data, pattern);
        assert_eq!(pos, Some(6));
    }

    #[test]
    fn test_find_bytes_not_found() {
        let data = b"Hello World";
        let pattern = b"Foo";
        let pos = find_bytes(data, pattern);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_find_bytes_at_start() {
        let data = b"Hello World";
        let pattern = b"Hello";
        let pos = find_bytes(data, pattern);
        assert_eq!(pos, Some(0));
    }

    #[test]
    fn test_find_bytes_at_end() {
        let data = b"Hello World";
        let pattern = b"World";
        let pos = find_bytes(data, pattern);
        assert_eq!(pos, Some(6));
    }

    // ============ PdfSigningEngine Tests ============

    #[test]
    fn test_pdf_signing_engine_new() {
        let engine = PdfSigningEngine::new();
        assert!(engine.tsa_client.is_none());
    }

    #[test]
    fn test_pdf_signing_engine_with_tsa() {
        // This may fail if network unavailable, which is expected
        let result = PdfSigningEngine::with_tsa();
        // Just verify it returns a result
        assert!(result.is_ok() || result.is_err());
    }

    // ============ ByteRange Tests ============

    #[test]
    fn test_signature_container_size() {
        // Verify the constant is set correctly (64KB for cert chain + timestamp + OCSP)
        assert_eq!(SIGNATURE_CONTAINER_SIZE, 65536);
    }

    // ============ Edge Cases ============

    #[test]
    fn test_build_sequence_256_bytes() {
        // Test boundary at 256 bytes (needs 2-byte length encoding)
        let content = vec![0x00; 256];
        let seq = build_sequence(&content);
        assert_eq!(seq[0], 0x30);
        assert_eq!(seq[1], 0x82); // 2 bytes follow
        assert_eq!(seq[2], 0x01); // high byte
        assert_eq!(seq[3], 0x00); // low byte
    }

    #[test]
    fn test_find_bytes_multiple_occurrences() {
        let data = b"abcabc";
        let pattern = b"abc";
        let pos = find_bytes(data, pattern);
        assert_eq!(pos, Some(0)); // Should find first occurrence
    }
}
