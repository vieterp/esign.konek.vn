/// Enhanced PDF parsing diagnostic tool
/// Tests the entire PDF signing flow to identify where the error occurs
use std::fs;
use std::path::Path;
use lopdf::{Dictionary, Document, Object};

fn main() {
    let pdf_path = "/Users/vieterp/Downloads/1_C26TSV_00000073.pdf";

    println!("=== PDF Signing Flow Diagnostic Tool ===");
    println!("Testing file: {}", pdf_path);
    println!();

    // Check file exists
    if !Path::new(pdf_path).exists() {
        eprintln!("ERROR: File does not exist at path: {}", pdf_path);
        std::process::exit(1);
    }

    // Read file
    let bytes = match fs::read(pdf_path) {
        Ok(b) => {
            println!("✓ File read successfully: {} bytes", b.len());
            b
        }
        Err(e) => {
            eprintln!("✗ Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    // Test 1: Load document
    println!("\n--- Test 1: Load PDF document ---");
    let mut doc = match Document::load_mem(&bytes) {
        Ok(d) => {
            println!("✓ Document loaded successfully");
            println!("  Version: {}", d.version);
            println!("  Pages: {}", d.get_pages().len());
            println!("  Objects: {}", d.objects.len());
            d
        }
        Err(e) => {
            eprintln!("✗ Failed to load document");
            eprintln!("  Error: {:#?}", e);
            std::process::exit(1);
        }
    };

    // Test 2: Get catalog
    println!("\n--- Test 2: Get PDF catalog ---");
    match doc.catalog() {
        Ok(_catalog) => {
            println!("✓ Catalog retrieved successfully");
        }
        Err(e) => {
            eprintln!("✗ Failed to get catalog");
            eprintln!("  Error: {:#?}", e);
            std::process::exit(1);
        }
    };

    // Test 3: Get mutable catalog
    println!("\n--- Test 3: Get mutable PDF catalog ---");
    match doc.catalog_mut() {
        Ok(_catalog) => {
            println!("✓ Mutable catalog retrieved successfully");
        }
        Err(e) => {
            eprintln!("✗ Failed to get mutable catalog");
            eprintln!("  Error: {:#?}", e);
            std::process::exit(1);
        }
    };

    // Test 4: Add AcroForm (minimal)
    println!("\n--- Test 4: Add AcroForm to catalog ---");
    let mut acro_form = Dictionary::new();
    acro_form.set("Fields", Object::Array(vec![]));
    acro_form.set("SigFlags", Object::Integer(3));

    let acro_form_id = doc.add_object(Object::Dictionary(acro_form));
    println!("✓ Created AcroForm object: {:?}", acro_form_id);

    match doc.catalog_mut() {
        Ok(catalog) => {
            catalog.set("AcroForm", Object::Reference(acro_form_id));
            println!("✓ Added AcroForm to catalog");
        }
        Err(e) => {
            eprintln!("✗ Failed to add AcroForm to catalog");
            eprintln!("  Error: {:#?}", e);
            std::process::exit(1);
        }
    }

    // Test 5: Create signature dictionary
    println!("\n--- Test 5: Create signature dictionary ---");
    let mut sig_dict = Dictionary::new();
    sig_dict.set("Type", Object::Name(b"Sig".to_vec()));
    sig_dict.set("Filter", Object::Name(b"Adobe.PPKLite".to_vec()));
    sig_dict.set("SubFilter", Object::Name(b"adbe.pkcs7.detached".to_vec()));

    let placeholder = vec![0u8; 65536];
    sig_dict.set(
        "Contents",
        Object::String(placeholder, lopdf::StringFormat::Hexadecimal),
    );

    sig_dict.set(
        "ByteRange",
        Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
        ]),
    );

    let sig_id = doc.add_object(Object::Dictionary(sig_dict));
    println!("✓ Created signature dictionary: {:?}", sig_id);

    // Test 6: Create widget
    println!("\n--- Test 6: Create signature widget ---");
    let mut widget = Dictionary::new();
    widget.set("Type", Object::Name(b"Annot".to_vec()));
    widget.set("Subtype", Object::Name(b"Widget".to_vec()));
    widget.set("FT", Object::Name(b"Sig".to_vec()));
    widget.set("Rect", Object::Array(vec![
        Object::Real(50.0),
        Object::Real(50.0),
        Object::Real(200.0),
        Object::Real(100.0),
    ]));
    widget.set("V", Object::Reference(sig_id));
    widget.set("T", Object::String(b"Signature1".to_vec(), lopdf::StringFormat::Literal));
    widget.set("F", Object::Integer(4)); // Print flag

    let widget_id = doc.add_object(Object::Dictionary(widget));
    println!("✓ Created widget: {:?}", widget_id);

    // Test 7: Save to buffer
    println!("\n--- Test 7: Save document to buffer ---");
    let mut output = Vec::new();
    match doc.save_to(&mut output) {
        Ok(_) => {
            println!("✓ Document saved successfully");
            println!("  Output size: {} bytes", output.len());
        }
        Err(e) => {
            eprintln!("✗ Failed to save document (IO Error)");
            eprintln!("  Error type: {:?}", std::any::type_name_of_val(&e));
            eprintln!("  Error kind: {:?}", e.kind());
            eprintln!("  Display: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== ALL TESTS PASSED ===");
}
