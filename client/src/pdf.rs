pub fn extract_pdf_text(filename: &str) -> String {
    let bytes = std::fs::read(filename).unwrap();
    pdf_extract::extract_text_from_mem(&bytes).unwrap()
}
