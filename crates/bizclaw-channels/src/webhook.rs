//! Webhook channel — receive inbound HTTP webhooks and send outbound.
//!
//! Useful for integrating with external systems (Zapier, n8n, custom APIs).

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage, ThreadType};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Webhook channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// URL to send outbound messages to.
    pub outbound_url: Option<String>,
    /// Secret for verifying inbound webhooks.
    pub secret: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// Webhook channel.
pub struct WebhookChannel {
    config: WebhookConfig,
    client: reqwest::Client,
    connected: bool,
    /// Sender for injecting inbound messages.
    inbound_tx: mpsc::UnboundedSender<IncomingMessage>,
    _inbound_rx: Option<mpsc::UnboundedReceiver<IncomingMessage>>,
}

impl WebhookChannel {
    pub fn new(config: WebhookConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            config,
            client: reqwest::Client::new(),
            connected: false,
            inbound_tx: tx,
            _inbound_rx: Some(rx),
        }
    }

    /// Inject an inbound message (called from HTTP handler).
    pub fn inject_message(&self, msg: IncomingMessage) -> Result<()> {
        self.inbound_tx.send(msg)
            .map_err(|_| BizClawError::Channel("Webhook receiver closed".into()))
    }

    /// Parse and verify an inbound webhook payload.
    pub fn parse_inbound(&self, payload: &str, signature: Option<&str>) -> Result<IncomingMessage> {
        // Verify signature if secret is configured
        if let (Some(secret), Some(sig)) = (&self.config.secret, signature) {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(format!("{secret}{payload}"));
            let expected = format!("{:x}", hasher.finalize());
            if expected != sig {
                return Err(BizClawError::AuthFailed("Invalid webhook signature".into()));
            }
        }

        let json: serde_json::Value = serde_json::from_str(payload)
            .map_err(|e| BizClawError::Channel(format!("Invalid webhook JSON: {e}")))?;

        Ok(IncomingMessage {
            channel: "webhook".into(),
            thread_id: json["thread_id"].as_str().unwrap_or("webhook").into(),
            sender_id: json["sender_id"].as_str().unwrap_or("external").into(),
            sender_name: json["sender_name"].as_str().map(String::from),
            content: json["content"].as_str().unwrap_or("").into(),
            thread_type: ThreadType::Direct,
            timestamp: chrono::Utc::now(),
            reply_to: None,
        })
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn name(&self) -> &str { "webhook" }

    async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        tracing::info!("Webhook channel connected");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        if let Some(url) = &self.config.outbound_url {
            let body = serde_json::json!({
                "thread_id": message.thread_id,
                "content": message.content,
                "reply_to": message.reply_to,
            });

            self.client.post(url)
                .json(&body)
                .send()
                .await
                .map_err(|e| BizClawError::Channel(format!("Webhook send failed: {e}")))?;
        }
        Ok(())
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        Ok(Box::new(stream::pending()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inbound_no_secret() {
        let channel = WebhookChannel::new(WebhookConfig {
            outbound_url: None,
            secret: None,
            enabled: true,
        });

        let payload = r#"{"content":"hello","sender_id":"user1","thread_id":"t1"}"#;
        let msg = channel.parse_inbound(payload, None).unwrap();
        assert_eq!(msg.content, "hello");
        assert_eq!(msg.sender_id, "user1");
        assert_eq!(msg.channel, "webhook");
    }
}
