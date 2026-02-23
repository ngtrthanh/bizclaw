//! Context management for agent conversations.
//!
//! Handles conversation history, context window limits,
//! and message summarization when context grows too large.

use bizclaw_core::types::Message;

/// Manages conversation context with window limits.
pub struct ConversationContext {
    messages: Vec<Message>,
    max_messages: usize,
    max_tokens_estimate: usize,
}

impl ConversationContext {
    /// Create a new conversation context.
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages,
            max_tokens_estimate: 0,
        }
    }

    /// Add a message to the context.
    pub fn push(&mut self, message: Message) {
        let token_estimate = message.content.len() / 4; // rough estimate: 4 chars per token
        self.max_tokens_estimate += token_estimate;
        self.messages.push(message);
        self.trim_if_needed();
    }

    /// Get all messages in context.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Clear all messages except the system prompt (first message).
    pub fn clear(&mut self) {
        if !self.messages.is_empty() {
            let system = self.messages[0].clone();
            self.messages.clear();
            self.messages.push(system);
        }
        self.max_tokens_estimate = 0;
    }

    /// Trim old messages if we exceed the maximum.
    /// Keeps the system prompt and the most recent messages.
    fn trim_if_needed(&mut self) {
        if self.messages.len() > self.max_messages {
            // Keep system prompt (index 0) and last N-1 messages
            let keep = self.max_messages - 1;
            let start = self.messages.len() - keep;
            let system = self.messages[0].clone();
            let recent: Vec<Message> = self.messages[start..].to_vec();
            self.messages.clear();
            self.messages.push(system);
            self.messages.extend(recent);

            // Recalculate token estimate
            self.max_tokens_estimate = self.messages.iter().map(|m| m.content.len() / 4).sum();
        }
    }

    /// Get estimated token count.
    pub fn estimated_tokens(&self) -> usize {
        self.max_tokens_estimate
    }

    /// Number of messages in context.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if context is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new(50) // Default: keep last 50 messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_push_and_access() {
        let mut ctx = ConversationContext::new(10);
        ctx.push(Message::system("You are helpful."));
        ctx.push(Message::user("Hello"));
        assert_eq!(ctx.len(), 2);
        assert_eq!(ctx.messages()[1].content, "Hello");
    }

    #[test]
    fn test_context_trim() {
        let mut ctx = ConversationContext::new(3);
        ctx.push(Message::system("System"));
        ctx.push(Message::user("msg1"));
        ctx.push(Message::user("msg2"));
        ctx.push(Message::user("msg3"));

        // Should keep system + last 2
        assert_eq!(ctx.len(), 3);
        assert_eq!(ctx.messages()[0].content, "System");
        assert_eq!(ctx.messages()[2].content, "msg3");
    }

    #[test]
    fn test_context_clear() {
        let mut ctx = ConversationContext::new(10);
        ctx.push(Message::system("System"));
        ctx.push(Message::user("Hello"));
        ctx.push(Message::assistant("Hi"));
        ctx.clear();
        assert_eq!(ctx.len(), 1);
        assert_eq!(ctx.messages()[0].content, "System");
    }

    #[test]
    fn test_estimated_tokens() {
        let mut ctx = ConversationContext::new(10);
        ctx.push(Message::user("abcdefgh")); // 8 chars = ~2 tokens
        assert!(ctx.estimated_tokens() > 0);
    }
}
