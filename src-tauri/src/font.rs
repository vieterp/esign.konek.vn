//! Font Embedding Module
//!
//! Handles embedding TrueType fonts in PDF for Vietnamese text support.
//! Creates Type 0 font structures with proper ToUnicode CMap for text extraction.

use flate2::write::ZlibEncoder;
use flate2::Compression;
use lopdf::{Dictionary, Object, ObjectId, Stream};
use std::io::Write;
use ttf_parser::Face;

/// Embedded Be Vietnam Pro Regular font (supports Vietnamese)
const BE_VIETNAM_PRO_REGULAR: &[u8] = include_bytes!("../fonts/BeVietnamPro-Regular.ttf");

/// Embedded Be Vietnam Pro SemiBold font (supports Vietnamese)
const BE_VIETNAM_PRO_SEMIBOLD: &[u8] = include_bytes!("../fonts/BeVietnamPro-SemiBold.ttf");

/// Font name used in PDF
const FONT_NAME: &str = "BeVietnamPro";
const FONT_NAME_BOLD: &str = "BeVietnamPro-SemiBold";

/// Result of embedding a font in a PDF document
pub struct EmbeddedFont {
    /// Object ID of the Type 0 font dictionary
    pub font_id: ObjectId,
}

/// Embed Vietnamese-capable font into PDF document
/// Returns the font object ID for use in content streams
pub fn embed_vietnamese_font(
    doc: &mut lopdf::Document,
    _resource_name: &str,
) -> Result<EmbeddedFont, String> {
    embed_font_data(doc, BE_VIETNAM_PRO_REGULAR, FONT_NAME)
}

/// Embed Vietnamese-capable bold font into PDF document
/// Returns the font object ID for use in content streams
pub fn embed_vietnamese_font_bold(
    doc: &mut lopdf::Document,
    _resource_name: &str,
) -> Result<EmbeddedFont, String> {
    embed_font_data(doc, BE_VIETNAM_PRO_SEMIBOLD, FONT_NAME_BOLD)
}

/// Internal function to embed font data
fn embed_font_data(
    doc: &mut lopdf::Document,
    font_data: &[u8],
    font_name: &str,
) -> Result<EmbeddedFont, String> {
    // 1. Compress TTF data
    let compressed_ttf = compress_data(font_data)?;

    // 2. Create FontFile2 stream (embedded TTF)
    let mut fontfile_dict = Dictionary::new();
    fontfile_dict.set("Length1", Object::Integer(font_data.len() as i64));
    fontfile_dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));

    let fontfile_stream = Stream::new(fontfile_dict, compressed_ttf);
    let fontfile_id = doc.add_object(Object::Stream(fontfile_stream));

    // 3. Create FontDescriptor
    let font_descriptor = create_font_descriptor(fontfile_id, font_name);
    let font_descriptor_id = doc.add_object(Object::Dictionary(font_descriptor));

    // 4. Create CIDFont dictionary
    let cid_font = create_cid_font(font_descriptor_id, font_name, font_data);
    let cid_font_id = doc.add_object(Object::Dictionary(cid_font));

    // 5. Create ToUnicode CMap stream
    let to_unicode_cmap = create_to_unicode_cmap();
    let mut cmap_dict = Dictionary::new();
    cmap_dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));
    let compressed_cmap = compress_data(to_unicode_cmap.as_bytes())?;
    let cmap_stream = Stream::new(cmap_dict, compressed_cmap);
    let cmap_id = doc.add_object(Object::Stream(cmap_stream));

    // 6. Create Type 0 font dictionary
    let type0_font = create_type0_font(cid_font_id, cmap_id, font_name);
    let font_id = doc.add_object(Object::Dictionary(type0_font));

    Ok(EmbeddedFont { font_id })
}

/// Create FontDescriptor dictionary
fn create_font_descriptor(fontfile_id: ObjectId, font_name: &str) -> Dictionary {
    let mut fd = Dictionary::new();
    fd.set("Type", Object::Name(b"FontDescriptor".to_vec()));
    fd.set(
        "FontName",
        Object::Name(format!("{}+{}", "AAAAAA", font_name).into_bytes()),
    );
    fd.set("Flags", Object::Integer(32)); // Symbolic
    fd.set(
        "FontBBox",
        Object::Array(vec![
            Object::Integer(-620),
            Object::Integer(-400),
            Object::Integer(2800),
            Object::Integer(1200),
        ]),
    );
    fd.set("ItalicAngle", Object::Integer(0));
    fd.set("Ascent", Object::Integer(1069));
    fd.set("Descent", Object::Integer(-293));
    fd.set("CapHeight", Object::Integer(714));
    fd.set("StemV", Object::Integer(88));
    fd.set("FontFile2", Object::Reference(fontfile_id));
    fd
}

/// Create CIDFont dictionary (CIDFontType2 for TrueType)
fn create_cid_font(font_descriptor_id: ObjectId, font_name: &str, font_data: &[u8]) -> Dictionary {
    let mut cidfont = Dictionary::new();
    cidfont.set("Type", Object::Name(b"Font".to_vec()));
    cidfont.set("Subtype", Object::Name(b"CIDFontType2".to_vec()));
    cidfont.set(
        "BaseFont",
        Object::Name(format!("{}+{}", "AAAAAA", font_name).into_bytes()),
    );

    // CIDSystemInfo
    let mut cid_system_info = Dictionary::new();
    cid_system_info.set(
        "Registry",
        Object::String(b"Adobe".to_vec(), lopdf::StringFormat::Literal),
    );
    cid_system_info.set(
        "Ordering",
        Object::String(b"Identity".to_vec(), lopdf::StringFormat::Literal),
    );
    cid_system_info.set("Supplement", Object::Integer(0));
    cidfont.set("CIDSystemInfo", Object::Dictionary(cid_system_info));

    cidfont.set("FontDescriptor", Object::Reference(font_descriptor_id));

    // Default width (fallback)
    cidfont.set("DW", Object::Integer(600));

    // Build W array with actual glyph widths from the font
    if let Ok(face) = Face::parse(font_data, 0) {
        let units_per_em = face.units_per_em() as f64;
        let scale = 1000.0 / units_per_em;

        // Build width array for common glyphs (0-500)
        let mut w_array: Vec<Object> = Vec::new();
        let mut i = 0u16;
        while i < 500 {
            if let Some(glyph_id) = ttf_parser::GlyphId(i).into() {
                if let Some(advance) = face.glyph_hor_advance(glyph_id) {
                    let width = (advance as f64 * scale).round() as i64;
                    // Format: [gid [width]]
                    w_array.push(Object::Integer(i as i64));
                    w_array.push(Object::Array(vec![Object::Integer(width)]));
                }
            }
            i += 1;
        }

        if !w_array.is_empty() {
            cidfont.set("W", Object::Array(w_array));
        }
    }

    // CIDToGIDMap - Identity mapping for TrueType
    cidfont.set("CIDToGIDMap", Object::Name(b"Identity".to_vec()));

    cidfont
}

/// Create Type 0 (composite) font dictionary
fn create_type0_font(
    cid_font_id: ObjectId,
    to_unicode_id: ObjectId,
    font_name: &str,
) -> Dictionary {
    let mut font = Dictionary::new();
    font.set("Type", Object::Name(b"Font".to_vec()));
    font.set("Subtype", Object::Name(b"Type0".to_vec()));
    font.set(
        "BaseFont",
        Object::Name(format!("{}+{}", "AAAAAA", font_name).into_bytes()),
    );
    font.set("Encoding", Object::Name(b"Identity-H".to_vec()));
    font.set(
        "DescendantFonts",
        Object::Array(vec![Object::Reference(cid_font_id)]),
    );
    font.set("ToUnicode", Object::Reference(to_unicode_id));
    font
}

/// Create ToUnicode CMap for identity mapping
/// This allows PDF readers to extract text properly
fn create_to_unicode_cmap() -> String {
    // Identity CMap - maps character codes directly to Unicode codepoints
    r#"/CIDInit /ProcSet findresource begin
12 dict begin
begincmap
/CIDSystemInfo
<< /Registry (Adobe)
/Ordering (UCS)
/Supplement 0
>> def
/CMapName /Adobe-Identity-UCS def
/CMapType 2 def
1 begincodespacerange
<0000> <FFFF>
endcodespacerange
1 beginbfrange
<0000> <FFFF> <0000>
endbfrange
endcmap
CMapName currentdict /CMap defineresource pop
end
end"#
        .to_string()
}

/// Compress data using zlib/deflate
fn compress_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| format!("Compression error: {}", e))?;
    encoder
        .finish()
        .map_err(|e| format!("Compression finish error: {}", e))
}

/// Convert UTF-8 string to PDF hex string using Glyph IDs from the regular font
/// This parses the font's cmap table to map Unicode → Glyph ID
pub fn utf8_to_pdf_hex(text: &str) -> String {
    utf8_to_pdf_hex_with_font(text, BE_VIETNAM_PRO_REGULAR)
}

/// Convert UTF-8 string to PDF hex string using Glyph IDs from the bold font
pub fn utf8_to_pdf_hex_bold(text: &str) -> String {
    utf8_to_pdf_hex_with_font(text, BE_VIETNAM_PRO_SEMIBOLD)
}

/// Internal function to convert UTF-8 to PDF hex using specified font
fn utf8_to_pdf_hex_with_font(text: &str, font_data: &[u8]) -> String {
    // Parse the embedded font
    let face = match Face::parse(font_data, 0) {
        Ok(f) => f,
        Err(_) => {
            // Fallback: return empty or use Unicode directly
            return text.encode_utf16().map(|c| format!("{:04X}", c)).collect();
        }
    };

    // Convert each character to its glyph ID
    let mut hex = String::new();
    for ch in text.chars() {
        let glyph_id = face.glyph_index(ch).map(|g| g.0).unwrap_or(0);
        hex.push_str(&format!("{:04X}", glyph_id));
    }
    hex
}

/// Parse hex color string (#RRGGBB) to RGB values (0.0-1.0)
pub fn parse_color_rgb(color: &str) -> (f64, f64, f64) {
    let color = color.trim_start_matches('#');
    if color.len() != 6 {
        return (0.0, 0.0, 0.0); // Default to black
    }

    let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0) as f64 / 255.0;

    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_to_pdf_hex_ascii() {
        let hex = utf8_to_pdf_hex("Hello");
        // Should produce glyph IDs, not Unicode
        assert!(!hex.is_empty());
        // Glyph IDs for "Hello" in Be Vietnam Pro
        println!("Hello glyph hex: {}", hex);
    }

    #[test]
    fn test_utf8_to_pdf_hex_vietnamese() {
        // "Được" in Vietnamese
        let hex = utf8_to_pdf_hex("Được");
        assert!(!hex.is_empty());
        println!("Được glyph hex: {}", hex);
    }

    #[test]
    fn test_utf8_to_pdf_hex_bold() {
        // Bold version should also work
        let hex = utf8_to_pdf_hex_bold("Hello");
        assert!(!hex.is_empty());
        println!("Hello bold glyph hex: {}", hex);
    }

    #[test]
    fn test_parse_color_rgb() {
        let (r, g, b) = parse_color_rgb("#FF0000");
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.0).abs() < 0.01);
        assert!((b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color_rgb_red() {
        let (r, g, b) = parse_color_rgb("#dc2626");
        assert!((r - 0.863).abs() < 0.01);
        assert!((g - 0.149).abs() < 0.01);
        assert!((b - 0.149).abs() < 0.01);
    }

    #[test]
    fn test_be_vietnam_pro_embedded() {
        // Verify font data is embedded
        assert!(!BE_VIETNAM_PRO_REGULAR.is_empty());
        assert!(!BE_VIETNAM_PRO_SEMIBOLD.is_empty());
        // Check TrueType magic bytes
        assert_eq!(&BE_VIETNAM_PRO_REGULAR[0..4], &[0x00, 0x01, 0x00, 0x00]);
        assert_eq!(&BE_VIETNAM_PRO_SEMIBOLD[0..4], &[0x00, 0x01, 0x00, 0x00]);
    }

    #[test]
    fn test_font_parsing() {
        let face = Face::parse(BE_VIETNAM_PRO_REGULAR, 0).expect("Failed to parse font");
        // Check that we can get glyph IDs
        let glyph_a = face.glyph_index('A').expect("No glyph for A");
        assert!(glyph_a.0 > 0);

        // Check Vietnamese character
        let glyph_d = face.glyph_index('Đ');
        println!("Glyph ID for Đ: {:?}", glyph_d);
    }
}
