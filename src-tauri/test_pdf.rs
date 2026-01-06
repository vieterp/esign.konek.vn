use lopdf::Document;

fn main() {
    let pdf_path = "/Users/vieterp/Downloads/1_C26TSV_00000073.pdf";
    match Document::load(pdf_path) {
        Ok(doc) => {
            println!("PDF loaded successfully!");
            println!("Version: {}", doc.version);
            println!("Pages: {}", doc.get_pages().len());
        }
        Err(e) => {
            eprintln!("Error loading PDF: {:?}", e);
            eprintln!("Error type: {}", e);
        }
    }
}
