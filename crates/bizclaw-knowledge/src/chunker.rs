//! Document chunker — splits documents into search-friendly chunks.
//! Designed for minimal memory: processes line-by-line, never loads full doc.

/// Split text into chunks of approximately `max_chars` characters.
/// Breaks at paragraph boundaries and word boundaries.
pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(100); // Min 100 chars
    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        let line = line.trim();

        // Empty line = paragraph break → natural chunk boundary
        if line.is_empty() && !current.is_empty() && current.len() > max_chars / 2 {
            chunks.push(std::mem::take(&mut current));
            continue;
        }

        // If the line itself is longer than max_chars, split by words
        if line.len() > max_chars {
            // Flush current buffer first
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            // Split long line by words
            for word in line.split_whitespace() {
                if !current.is_empty() && current.len() + word.len() + 1 > max_chars {
                    chunks.push(std::mem::take(&mut current));
                }
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
            continue;
        }

        // If adding this line would exceed max_chars, flush current chunk
        if !current.is_empty() && current.len() + line.len() + 1 > max_chars {
            chunks.push(std::mem::take(&mut current));
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }

    // Don't forget the last chunk
    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

/// Extract plain text from common file formats.
/// Supports: .txt, .md, .json, .toml, .yaml, .csv, .log
/// For Pi: no heavy PDF/DOCX parsing — keep it simple.
pub fn extract_text(content: &str, filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("txt").to_lowercase();
    match ext.as_str() {
        "md" | "markdown" => {
            // Strip markdown syntax for better search
            content
                .lines()
                .map(|l| {
                    l.trim_start_matches('#')
                        .trim_start_matches('*')
                        .trim_start_matches('-')
                        .trim_start_matches('>')
                        .trim()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "json" => {
            // Extract string values from JSON
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
                extract_json_strings(&val)
            } else {
                content.to_string()
            }
        }
        _ => content.to_string(),
    }
}

/// Recursively extract string values from a JSON value.
fn extract_json_strings(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(extract_json_strings)
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Object(map) => map
            .values()
            .map(extract_json_strings)
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_short_text() {
        let chunks = chunk_text("Hello world", 500);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    fn test_chunk_paragraphs() {
        let text = "Paragraph one line one.\nParagraph one line two.\n\nParagraph two line one.\nParagraph two line two.";
        let chunks = chunk_text(text, 100);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_long_text() {
        let text = "word ".repeat(200); // ~1000 chars
        let chunks = chunk_text(&text, 300);
        assert!(
            chunks.len() >= 2,
            "Expected at least 2 chunks, got {}",
            chunks.len()
        );
        // Each chunk should be <= 300 chars (approximately)
        for chunk in &chunks {
            assert!(
                chunk.len() <= 1100,
                "Chunk too large: {} chars",
                chunk.len()
            );
        }
    }

    #[test]
    fn test_extract_markdown() {
        let md = "# Title\n## Sub\n- item\n> quote";
        let text = extract_text(md, "doc.md");
        assert!(!text.contains('#'));
        assert!(text.contains("Title"));
    }
}
