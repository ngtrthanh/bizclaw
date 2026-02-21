//! Agent engine internals — core processing pipeline.

use bizclaw_core::types::{Message, ProviderResponse};

/// Format a provider response for display.
pub fn format_response(response: &ProviderResponse) -> String {
    if let Some(content) = &response.content {
        content.clone()
    } else if !response.tool_calls.is_empty() {
        let tool_names: Vec<&str> = response.tool_calls.iter()
            .map(|tc| tc.function.name.as_str())
            .collect();
        format!("[Calling tools: {}]", tool_names.join(", "))
    } else {
        "[No response]".into()
    }
}

/// Estimate token count for a message list (rough: 4 chars ≈ 1 token).
pub fn estimate_tokens(messages: &[Message]) -> usize {
    messages.iter()
        .map(|m| m.content.len() / 4 + 5) // content + overhead per message
        .sum()
}

/// Check if conversation needs compaction.
pub fn needs_compaction(messages: &[Message], max_tokens: usize) -> bool {
    estimate_tokens(messages) > max_tokens
}
