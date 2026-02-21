use async_trait::async_trait;
use bizclaw_core::error::Result;
use bizclaw_core::traits::Tool;
use bizclaw_core::types::{ToolDefinition, ToolResult};
use regex::Regex;
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct DocumentReaderTool;

impl DocumentReaderTool {
    pub fn new() -> Self {
        Self
    }

    fn read_pdf(&self, path: &Path) -> Result<String> {
        match pdf_extract::extract_text(path) {
            Ok(text) => Ok(text),
            Err(e) => Err(bizclaw_core::error::BizClawError::Tool(format!(
                "Failed to parse PDF: {e}"
            ))),
        }
    }

    fn read_docx(&self, path: &Path) -> Result<String> {
        let file = fs::File::open(path).map_err(|e| bizclaw_core::error::BizClawError::Tool(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(format!("Invalid zip archive: {e}")))?;

        let mut xml_content = String::new();
        if let Ok(mut doc_file) = archive.by_name("word/document.xml") {
            doc_file.read_to_string(&mut xml_content).map_err(|e| bizclaw_core::error::BizClawError::Tool(e.to_string()))?;
        } else {
            return Err(bizclaw_core::error::BizClawError::Tool(
                "Not a valid DOCX file (missing word/document.xml)".into(),
            ));
        }

        let p_re = Regex::new(r"<w:p\b[^>]*>(.*?)</w:p>").unwrap();
        let t_re = Regex::new(r"<w:t\b[^>]*>(.*?)</w:t>").unwrap();
        
        let mut full_text = String::new();
        for p_cap in p_re.captures_iter(&xml_content) {
            if let Some(m) = p_cap.get(1) {
                let p_content = m.as_str();
                let mut line = String::new();
                for t_cap in t_re.captures_iter(p_content) {
                    if let Some(t_m) = t_cap.get(1) {
                        let text = t_m.as_str()
                            .replace("&lt;", "<")
                            .replace("&gt;", ">")
                            .replace("&amp;", "&")
                            .replace("&quot;", "\"")
                            .replace("&apos;", "'");
                        line.push_str(&text);
                    }
                }
                if !line.trim().is_empty() {
                    full_text.push_str(&line);
                    full_text.push('\n');
                }
            }
        }

        Ok(full_text)
    }

    fn read_excel(&self, path: &Path) -> Result<String> {
        use calamine::{open_workbook_auto, Reader, Data};
        
        let mut workbook = open_workbook_auto(path)
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(format!("Failed to open Excel: {e}")))?;
        
        let sheet_names = workbook.sheet_names().to_owned();
        let mut full_text = String::new();
        
        for sheet_name in sheet_names {
            full_text.push_str(&format!("--- Sheet: {} ---\n", sheet_name));
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                for row in range.rows() {
                    let cols: Vec<String> = row.iter().map(|cell| {
                        match cell {
                            Data::String(s) => s.to_string(),
                            Data::Float(f) => f.to_string(),
                            Data::Int(i) => i.to_string(),
                            Data::Bool(b) => b.to_string(),
                            Data::Empty => String::new(),
                            Data::Error(e) => format!("Error({e})"),
                            Data::DateTime(v) => v.as_f64().to_string(),
                            Data::DateTimeIso(v) => v.to_string(),
                            Data::DurationIso(v) => v.to_string(),
                        }
                    }).collect();
                    full_text.push_str(&cols.join("\t"));
                    full_text.push('\n');
                }
            }
        }
        
        Ok(full_text)
    }
}

impl Default for DocumentReaderTool {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Tool for DocumentReaderTool {
    fn name(&self) -> &str {
        "document_reader"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "document_reader".into(),
            description: "Extracts clean text from offline documents (PDF, DOCX, XLSX, TXT, CSV). VERY useful for analyzing contracts, reports, attachments, and files dropped in by the user securely without uploading to the cloud.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["read_file"],
                        "description": "Action to perform."
                    },
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the document file on disk."
                    }
                },
                "required": ["action", "path"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(e.to_string()))?;

        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        if action != "read_file" {
            return Err(bizclaw_core::error::BizClawError::Tool(
                "Invalid action for document_reader. Use 'read_file'".into(),
            ));
        }

        if path_str.is_empty() {
            return Err(bizclaw_core::error::BizClawError::Tool(
                "Missing 'path' argument".into(),
            ));
        }

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(bizclaw_core::error::BizClawError::Tool(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut content = match ext.as_str() {
            "pdf" => self.read_pdf(path)?,
            "docx" => self.read_docx(path)?,
            "xlsx" | "xls" | "csv" => self.read_excel(path)?,
            "txt" | "md" | "json" | "xml" | "rs" | "log" => {
                fs::read_to_string(path).map_err(|e| {
                    bizclaw_core::error::BizClawError::Tool(format!("Failed to read text file: {e}"))
                })?
            }
            _ => {
                return Err(bizclaw_core::error::BizClawError::Tool(format!(
                    "Unsupported file extension: {}",
                    ext
                )))
            }
        };

        let char_limit = 100_000;
        if content.len() > char_limit {
            content.truncate(char_limit);
            content.push_str("\n\n[... TEXT TRUNCATED DUE TO LENGTH LIMIT ...]");
        }

        Ok(ToolResult {
            tool_call_id: String::new(),
            output: format!("Extracted content from {}:\n\n{}", path.display(), content),
            success: true,
        })
    }
}
