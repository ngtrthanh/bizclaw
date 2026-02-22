//! Zalo messaging — send/receive text, images, stickers, files.
//! Based on Zalo Web HTTP API endpoints.

use bizclaw_core::error::{BizClawError, Result};
use serde::{Deserialize, Serialize};

/// Message types supported by Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    Sticker,
    File,
    Link,
    Location,
    Contact,
    Gif,
    Video,
}

/// Send message request.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub thread_id: String,
    pub thread_type: ThreadType,
    pub msg_type: MessageType,
    pub content: String,
    /// Optional quote/reply to a message
    pub quote_msg_id: Option<String>,
    /// Optional mention user IDs
    pub mentions: Vec<String>,
}

/// Thread type for Zalo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThreadType {
    /// Direct message (1:1)
    User = 0,
    /// Group chat
    Group = 1,
}

/// Zalo messaging client.
pub struct ZaloMessaging {
    client: reqwest::Client,
    base_url: String,
}

impl ZaloMessaging {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://tt-chat-wpa.chat.zalo.me/api".into(),
        }
    }

    /// Send a text message.
    pub async fn send_text(
        &self,
        thread_id: &str,
        thread_type: ThreadType,
        content: &str,
        cookie: &str,
    ) -> Result<String> {
        let endpoint = if thread_type == ThreadType::User {
            format!("{}/message/sms", self.base_url)
        } else {
            format!("{}/group/sendmsg", self.base_url)
        };

        let params = serde_json::json!({
            "toid": thread_id,
            "message": content,
            "clientId": generate_client_id(),
        });

        let response = self
            .client
            .post(&endpoint)
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Send message failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid send response: {e}")))?;

        let error_code = body["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(BizClawError::Channel(format!(
                "Send failed: {} - {}",
                error_code,
                body["error_message"].as_str().unwrap_or("unknown")
            )));
        }

        let msg_id = body["data"]["msgId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        tracing::debug!("Sent message {} to {}", msg_id, thread_id);
        Ok(msg_id)
    }

    /// Send a reaction to a message.
    pub async fn send_reaction(
        &self,
        msg_id: &str,
        thread_id: &str,
        reaction: &str,
        cookie: &str,
    ) -> Result<()> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
            "rType": reaction,
        });

        self.client
            .post(format!("{}/message/reaction", self.base_url))
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Reaction failed: {e}")))?;

        Ok(())
    }

    /// Undo (recall) a message.
    pub async fn undo_message(&self, msg_id: &str, thread_id: &str, cookie: &str) -> Result<()> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
        });

        self.client
            .post(format!("{}/message/undo", self.base_url))
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Undo message failed: {e}")))?;

        Ok(())
    }
}

impl Default for ZaloMessaging {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a client-side message ID.
fn generate_client_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.r#gen::<u64>() % 9_999_999_999;
    format!("cli_{}", id)
}
