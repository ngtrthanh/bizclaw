//! Web Search Tool — enables the agent to search the internet.
//!
//! Uses DuckDuckGo Instant Answer API (no API key required).

use async_trait::async_trait;
use bizclaw_core::error::Result;
use bizclaw_core::traits::Tool;
use bizclaw_core::types::{ToolDefinition, ToolResult};

pub struct WebSearchTool;

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_search".into(),
            description: "Search the web and return results".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "max_results": { "type": "integer", "description": "Max results (default 5)" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, arguments: &str) -> Result<ToolResult> {
        let args: serde_json::Value = serde_json::from_str(arguments)
            .unwrap_or_else(|_| serde_json::json!({"query": arguments}));

        let query = args["query"].as_str().unwrap_or(arguments);

        let max_results: usize = args["max_results"]
            .as_u64()
            .map(|v| v as usize)
            .unwrap_or(5);

        // Use DuckDuckGo HTML search (no API key needed)
        let client = reqwest::Client::builder()
            .user_agent("BizClaw/1.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(format!("HTTP error: {e}")))?;

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );
        let response =
            client.get(&url).send().await.map_err(|e| {
                bizclaw_core::error::BizClawError::Tool(format!("Search failed: {e}"))
            })?;

        let html = response
            .text()
            .await
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(format!("Read failed: {e}")))?;

        let results = parse_ddg_results(&html, max_results);

        let output = if results.is_empty() {
            format!("No results found for: {query}")
        } else {
            let mut out = format!("Search results for \"{query}\":\n\n");
            for (i, r) in results.iter().enumerate() {
                out.push_str(&format!("{}. {}\n   {}\n   {}\n\n", i + 1, r.0, r.1, r.2));
            }
            out
        };

        Ok(ToolResult {
            tool_call_id: String::new(),
            output,
            success: true,
        })
    }
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<(String, String, String)> {
    let mut results = Vec::new();

    for segment in html.split("class=\"result__a\"").skip(1).take(max) {
        let title = extract_between(segment, ">", "</a>")
            .unwrap_or_default()
            .replace("<b>", "")
            .replace("</b>", "");

        let url = extract_between(segment, "href=\"", "\"").unwrap_or_default();

        let snippet = if let Some(snip_seg) = segment.split("class=\"result__snippet\"").nth(1) {
            extract_between(snip_seg, ">", "</")
                .unwrap_or_default()
                .replace("<b>", "")
                .replace("</b>", "")
        } else {
            String::new()
        };

        if !title.is_empty() {
            results.push((
                title.trim().into(),
                snippet.trim().into(),
                url.trim().into(),
            ));
        }
    }
    results
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_idx = text.find(start)? + start.len();
    let remaining = &text[start_idx..];
    let end_idx = remaining.find(end)?;
    Some(remaining[..end_idx].to_string())
}
