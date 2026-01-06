/// Test path validation with the actual PDF file
use std::path::Path;

fn main() {
    let pdf_path = "/Users/vieterp/Downloads/1_C26TSV_00000073.pdf";

    println!("=== PDF Path Validation Test ===");
    println!("Testing path: {}", pdf_path);
    println!();

    let path = Path::new(pdf_path);

    // Test 1: Path exists
    println!("--- Test 1: Path exists ---");
    if path.exists() {
        println!("✓ Path exists");
        println!("  Is file: {}", path.is_file());
        println!("  Is dir: {}", path.is_dir());
    } else {
        eprintln!("✗ Path does not exist");
        std::process::exit(1);
    }

    // Test 2: Canonicalize (resolve symlinks, relative paths)
    println!("\n--- Test 2: Canonicalize path ---");
    match path.canonicalize() {
        Ok(canonical) => {
            println!("✓ Canonical path: {}", canonical.display());
        }
        Err(e) => {
            eprintln!("✗ Canonicalize failed");
            eprintln!("  Error: {}", e);
            eprintln!("  Error kind: {:?}", e.kind());
            std::process::exit(1);
        }
    }

    // Test 3: Extension check
    println!("\n--- Test 3: Extension check ---");
    let ext = path
        .extension()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    println!("  Extension (raw): {:?}", path.extension());
    println!("  Extension (lowercased): {:?}", ext);

    if ext == "pdf" {
        println!("✓ Valid PDF extension");
    } else {
        eprintln!("✗ Not a PDF file (extension: {:?})", ext);
        std::process::exit(1);
    }

    // Test 4: Read file
    println!("\n--- Test 4: Read file ---");
    match std::fs::read(path) {
        Ok(bytes) => {
            println!("✓ File read successfully");
            println!("  Size: {} bytes", bytes.len());

            // Check PDF header
            if bytes.starts_with(b"%PDF-") {
                let version = String::from_utf8_lossy(&bytes[5..8]);
                println!("  PDF header: %PDF-{}", version);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to read file");
            eprintln!("  Error: {}", e);
            eprintln!("  Error kind: {:?}", e.kind());
            std::process::exit(1);
        }
    }

    println!("\n=== ALL VALIDATION TESTS PASSED ===");
}
