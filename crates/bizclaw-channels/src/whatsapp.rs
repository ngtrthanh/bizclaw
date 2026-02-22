//! WhatsApp Business Cloud API channel.
//!
//! Uses the official WhatsApp Business Platform (Cloud API) for messaging.
//! Requires: Access Token + Phone Number ID from Meta Business Suite.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};

/// WhatsApp Business channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    /// Facebook Graph API access token
    pub access_token: String,
    /// WhatsApp Phone Number ID
    pub phone_number_id: String,
    /// Webhook verify token (for incoming messages)
    #[serde(default)]
    pub webhook_verify_token: String,
    /// Business Account ID (optional)
    #[serde(default)]
    pub business_id: String,
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            phone_number_id: String::new(),
            webhook_verify_token: String::new(),
            business_id: String::new(),
        }
    }
}

/// WhatsApp Business channel implementation.
pub struct WhatsAppChannel {
    config: WhatsAppConfig,
    client: reqwest::Client,
    connected: bool,
}

impl WhatsAppChannel {
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            connected: false,
        }
    }

    /// Send a text message via WhatsApp Cloud API.
    async fn send_text_message(&self, to: &str, text: &str) -> Result<String> {
        let url = format!(
            "https://graph.facebook.com/v21.0/{}/messages",
            self.config.phone_number_id
        );

        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": {
                "preview_url": false,
                "body": text
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("WhatsApp API request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(BizClawError::Channel(format!(
                "WhatsApp API error {}: {}", status, error_text
            )));
        }

        let result: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid WhatsApp response: {e}")))?;

        let msg_id = result["messages"][0]["id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        tracing::debug!("WhatsApp message sent: {} → {}", msg_id, to);
        Ok(msg_id)
    }

    /// Mark a message as read.
    pub async fn mark_as_read(&self, message_id: &str) -> Result<()> {
        let url = format!(
            "https://graph.facebook.com/v21.0/{}/messages",
            self.config.phone_number_id
        );

        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });

        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("WhatsApp mark-read failed: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str { "whatsapp" }

    async fn connect(&mut self) -> Result<()> {
        if self.config.access_token.is_empty() {
            return Err(BizClawError::Config(
                "WhatsApp access_token not configured".into()
            ));
        }
        if self.config.phone_number_id.is_empty() {
            return Err(BizClawError::Config(
                "WhatsApp phone_number_id not configured".into()
            ));
        }

        // Verify token by checking phone number
        let url = format!(
            "https://graph.facebook.com/v21.0/{}",
            self.config.phone_number_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.access_token))
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("WhatsApp verification failed: {e}")))?;

        if response.status().is_success() {
            self.connected = true;
            tracing::info!("WhatsApp Business: connected (phone_id={})", self.config.phone_number_id);
        } else {
            let text = response.text().await.unwrap_or_default();
            return Err(BizClawError::AuthFailed(format!(
                "WhatsApp token verification failed: {}", text
            )));
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        tracing::info!("WhatsApp Business: disconnected");
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        // WhatsApp incoming messages arrive via webhook (HTTP POST).
        // The webhook handler in bizclaw-gateway converts them to IncomingMessage.
        tracing::info!("WhatsApp: listening via webhook endpoint");
        Ok(Box::new(futures::stream::pending::<IncomingMessage>()))
    }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        self.send_text_message(&message.thread_id, &message.content).await?;
        Ok(())
    }

    async fn send_typing(&self, _thread_id: &str) -> Result<()> {
        // WhatsApp doesn't support typing indicators via Cloud API
        Ok(())
    }
}
