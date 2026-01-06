/// Minimal lopdf error diagnostic tool
/// Tests PDF parsing to identify exact error type and message
use std::fs;
use std::path::Path;

fn main() {
    let pdf_path = "/Users/vieterp/Downloads/1_C26TSV_00000073.pdf";

    println!("=== PDF Parsing Diagnostic Tool ===");
    println!("Testing file: {}", pdf_path);
    println!();

    // Check file exists
    if !Path::new(pdf_path).exists() {
        eprintln!("ERROR: File does not exist at path: {}", pdf_path);
        std::process::exit(1);
    }

    // Read file metadata
    match fs::metadata(pdf_path) {
        Ok(metadata) => {
            println!("File size: {} bytes", metadata.len());
        }
        Err(e) => {
            eprintln!("ERROR: Cannot read file metadata: {}", e);
            std::process::exit(1);
        }
    }

    // Test 1: Load from file path
    println!("\n--- Test 1: lopdf::Document::load() from file path ---");
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            println!("✓ SUCCESS: Document loaded successfully");
            println!("  Version: {}", doc.version);
            println!("  Number of pages: {}", doc.get_pages().len());
            println!("  Objects count: {}", doc.objects.len());
        }
        Err(e) => {
            println!("✗ FAILED: Document::load() error");
            println!("\n=== EXACT ERROR DETAILS ===");
            println!("Error type: {:?}", std::any::type_name_of_val(&e));
            println!("Debug format: {:#?}", e);
            println!("Display format: {}", e);

            // Pattern matching to identify error variant
            println!("\n=== ERROR VARIANT MATCH ===");
            match &e {
                lopdf::Error::Decryption(msg) => println!("Variant: Decryption({})", msg),
                lopdf::Error::NotEncrypted => println!("Variant: NotEncrypted"),
                lopdf::Error::AlreadyEncrypted => println!("Variant: AlreadyEncrypted"),
                lopdf::Error::UnsupportedSecurityHandler(msg) => println!("Variant: UnsupportedSecurityHandler({:?})", msg),
                lopdf::Error::ToUnicodeCMap(msg) => println!("Variant: ToUnicodeCMap({})", msg),
                lopdf::Error::Parse(msg) => println!("Variant: Parse({})", msg),
                lopdf::Error::Xref(msg) => println!("Variant: Xref({})", msg),
                lopdf::Error::InvalidObjectStream(msg) => println!("Variant: InvalidObjectStream({})", msg),
                lopdf::Error::InvalidStream(msg) => println!("Variant: InvalidStream({})", msg),
                lopdf::Error::Decompress(msg) => println!("Variant: Decompress({})", msg),
                _ => println!("Variant: Other/Unknown - {}", e),
            }
        }
    }

    // Test 2: Load from memory
    println!("\n--- Test 2: lopdf::Document::load_mem() from bytes ---");
    match fs::read(pdf_path) {
        Ok(bytes) => {
            println!("Read {} bytes into memory", bytes.len());

            match lopdf::Document::load_mem(&bytes) {
                Ok(doc) => {
                    println!("✓ SUCCESS: Document loaded from memory successfully");
                    println!("  Version: {}", doc.version);
                    println!("  Number of pages: {}", doc.get_pages().len());
                }
                Err(e) => {
                    println!("✗ FAILED: Document::load_mem() error");
                    println!("\n=== EXACT ERROR DETAILS ===");
                    println!("Error type: {:?}", std::any::type_name_of_val(&e));
                    println!("Debug format: {:#?}", e);
                    println!("Display format: {}", e);

                    println!("\n=== ERROR VARIANT MATCH ===");
                    match &e {
                        lopdf::Error::Decryption(msg) => println!("Variant: Decryption({})", msg),
                        lopdf::Error::NotEncrypted => println!("Variant: NotEncrypted"),
                        lopdf::Error::AlreadyEncrypted => println!("Variant: AlreadyEncrypted"),
                        lopdf::Error::UnsupportedSecurityHandler(msg) => println!("Variant: UnsupportedSecurityHandler({:?})", msg),
                        lopdf::Error::ToUnicodeCMap(msg) => println!("Variant: ToUnicodeCMap({})", msg),
                        lopdf::Error::Parse(msg) => println!("Variant: Parse({})", msg),
                        lopdf::Error::Xref(msg) => println!("Variant: Xref({})", msg),
                        lopdf::Error::InvalidObjectStream(msg) => println!("Variant: InvalidObjectStream({})", msg),
                        lopdf::Error::InvalidStream(msg) => println!("Variant: InvalidStream({})", msg),
                        lopdf::Error::Decompress(msg) => println!("Variant: Decompress({})", msg),
                        _ => println!("Variant: Other/Unknown - {}", e),
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("ERROR: Cannot read file into memory: {}", e);
        }
    }

    println!("\n=== DIAGNOSTIC COMPLETE ===");
}
