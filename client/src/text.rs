use regex::Regex;

#[derive(PartialEq, Debug, Clone)]
pub struct TextSummary {
    text: String,
}

impl TextSummary {
    pub fn new(text: String) -> TextSummary {
        TextSummary { text }
    }

    pub fn replace_escape_sequences(&mut self) {
        let escape_pattern = Regex::new(r#"\s+"#).unwrap();
        let replaced_text = escape_pattern.replace_all(self.text.as_str(), " ");
        self.text = replaced_text.to_string();
    }

    pub fn tokenize_words_into_chunks(
        &mut self,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Vec<String> {
        self.replace_escape_sequences();

        let tokens: Vec<_> = self.text.split_whitespace().map(String::from).collect();
        let mut buckets = Vec::new();
        for i in (0..tokens.len()).step_by(chunk_size - chunk_overlap) {
            let bucket_end = i + chunk_size;
            if bucket_end <= tokens.len() {
                buckets.push(tokens[i..bucket_end].to_vec().join(" "));
            }
        }
        buckets
    }
}

pub fn extract_pdf_text(filename: &str) -> String {
    let bytes = std::fs::read(filename).unwrap();
    pdf_extract::extract_text_from_mem(&bytes).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_summary_1() {
        let str_1 = String::from("A B C D E F G");
        let mut summary: TextSummary = TextSummary::new(str_1);
        summary.tokenize_words_into_chunks(3, 1);
        let chunks = summary.tokenize_words_into_chunks(3, 1);
        let expected = vec![
            "A B C".to_string(),
            "C D E".to_string(),
            "E F G".to_string(),
        ];
        assert_eq!(expected, chunks);
    }

    #[test]
    fn test_text_summary_2() {
        let contents = extract_pdf_text("test/files/2401.14149.pdf");
        assert!(contents.len() > 0);
    }
}
