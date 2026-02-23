//! Zalo OA Bot — Official Bot Platform API.
//! Based on https://bot.zapps.me/docs/
//!
//! This is DIFFERENT from zca-js (unofficial personal API).
//! Uses Bot Token authentication and bot-api.zaloplatforms.com endpoint.
//! Supports both Long Polling (getUpdates) and Webhook (setWebhook) modes.

use serde::{Deserialize, Serialize};
use bizclaw_core::error::{BizClawError, Result};

/// Base URL for Zalo Bot API.
const BOT_API_BASE: &str = "https://bot-api.zaloplatforms.com";

// ─── Types ────────────────────────────────────────────

/// Bot info from getMe API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    pub id: String,
    pub name: String,
    pub username: Option<String>,
}

/// Incoming update from getUpdates or Webhook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotUpdate {
    pub update_id: Option<i64>,
    pub message: Option<BotMessage>,
}

/// A message received by the bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMessage {
    /// Message ID
    pub message_id: String,
    /// Sender info
    pub from: Option<BotUser>,
    /// Chat info (user or group)
    pub chat: BotChat,
    /// Message text
    pub text: Option<String>,
    /// Timestamp
    pub date: Option<i64>,
    /// Reply to message
    pub reply_to_message: Option<Box<BotMessage>>,
}

/// User info in bot context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotUser {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}

/// Chat info (can be user or group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotChat {
    pub id: String,
    /// "private" or "group"
    #[serde(rename = "type")]
    pub chat_type: Option<String>,
    pub title: Option<String>,
}

/// Webhook info from getWebhookInfo API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookInfo {
    pub url: String,
    pub updated_at: Option<i64>,
}

/// Send message result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub message_id: String,
    pub date: Option<i64>,
}

// ─── Client ────────────────────────────────────────────

/// Zalo OA Bot client.
/// API docs: https://bot.zapps.me/docs/
pub struct ZaloBotClient {
    client: reqwest::Client,
    bot_token: String,
    /// Last update_id for long polling
    last_update_id: Option<i64>,
}

impl ZaloBotClient {
    /// Create a new bot client with the given token.
    /// Token is obtained from Zalo Bot Creator MiniApp.
    pub fn new(bot_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            bot_token: bot_token.to_string(),
            last_update_id: None,
        }
    }

    /// Build API URL for a method.
    fn api_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", BOT_API_BASE, self.bot_token, method)
    }

    // ─── Bot Info ───────────────────────────────────

    /// Get bot info.
    /// API: GET /bot{TOKEN}/getMe
    pub async fn get_me(&self) -> Result<BotInfo> {
        let url = self.api_url("getMe");
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("getMe failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("getMe parse error: {e}")))?;

        if !body["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "getMe error: {}",
                body["description"].as_str().unwrap_or("unknown")
            )));
        }

        let result = &body["result"];
        Ok(BotInfo {
            id: result["id"].as_str().unwrap_or("").into(),
            name: result["name"].as_str().unwrap_or("").into(),
            username: result["username"].as_str().map(String::from),
        })
    }

    // ─── Messaging ─────────────────────────────────

    /// Send a text message.
    /// API: POST /bot{TOKEN}/sendMessage
    /// Body: { chat_id, text }
    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<SendResult> {
        let url = self.api_url("sendMessage");

        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("sendMessage failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("sendMessage parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "sendMessage error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        let result = &resp["result"];
        Ok(SendResult {
            message_id: result["message_id"].as_str().unwrap_or("").into(),
            date: result["date"].as_i64(),
        })
    }

    /// Send a photo.
    /// API: POST /bot{TOKEN}/sendPhoto
    /// Body: { chat_id, photo (URL), caption? }
    pub async fn send_photo(
        &self,
        chat_id: &str,
        photo_url: &str,
        caption: Option<&str>,
    ) -> Result<SendResult> {
        let url = self.api_url("sendPhoto");

        let mut body = serde_json::json!({
            "chat_id": chat_id,
            "photo": photo_url,
        });

        if let Some(cap) = caption {
            body["caption"] = serde_json::Value::String(cap.to_string());
        }

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("sendPhoto failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("sendPhoto parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "sendPhoto error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        let result = &resp["result"];
        Ok(SendResult {
            message_id: result["message_id"].as_str().unwrap_or("").into(),
            date: result["date"].as_i64(),
        })
    }

    // ─── Updates (Long Polling) ────────────────────

    /// Get updates via long polling.
    /// API: POST /bot{TOKEN}/getUpdates
    /// Body: { timeout? }
    ///
    /// NOTE: Does NOT work if webhook is set. Call deleteWebhook first.
    pub async fn get_updates(&mut self, timeout: Option<u32>) -> Result<Vec<BotUpdate>> {
        let url = self.api_url("getUpdates");

        let mut body = serde_json::json!({});
        if let Some(t) = timeout {
            body["timeout"] = serde_json::Value::Number(t.into());
        }
        if let Some(offset) = self.last_update_id {
            body["offset"] = serde_json::Value::Number((offset + 1).into());
        }

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("getUpdates failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("getUpdates parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "getUpdates error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        let updates: Vec<BotUpdate> = serde_json::from_value(
            resp["result"].clone()
        ).unwrap_or_default();

        // Track last update ID for polling
        if let Some(last) = updates.last() {
            if let Some(id) = last.update_id {
                self.last_update_id = Some(id);
            }
        }

        Ok(updates)
    }

    // ─── Webhook ──────────────────────────────────

    /// Set webhook URL.
    /// API: POST /bot{TOKEN}/setWebhook
    /// Body: { url, secret_token? }
    pub async fn set_webhook(
        &self,
        webhook_url: &str,
        secret_token: Option<&str>,
    ) -> Result<WebhookInfo> {
        let url = self.api_url("setWebhook");

        let mut body = serde_json::json!({
            "url": webhook_url,
        });

        if let Some(token) = secret_token {
            body["secret_token"] = serde_json::Value::String(token.to_string());
        }

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("setWebhook failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("setWebhook parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "setWebhook error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        let result = &resp["result"];
        Ok(WebhookInfo {
            url: result["url"].as_str().unwrap_or("").into(),
            updated_at: result["updated_at"].as_i64(),
        })
    }

    /// Delete webhook.
    /// API: POST /bot{TOKEN}/deleteWebhook
    pub async fn delete_webhook(&self) -> Result<()> {
        let url = self.api_url("deleteWebhook");

        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("deleteWebhook failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("deleteWebhook parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "deleteWebhook error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        Ok(())
    }

    /// Get current webhook info.
    /// API: GET /bot{TOKEN}/getWebhookInfo
    pub async fn get_webhook_info(&self) -> Result<WebhookInfo> {
        let url = self.api_url("getWebhookInfo");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("getWebhookInfo failed: {e}")))?;

        let resp: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("getWebhookInfo parse error: {e}")))?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(BizClawError::Channel(format!(
                "getWebhookInfo error: {}",
                resp["description"].as_str().unwrap_or("unknown")
            )));
        }

        let result = &resp["result"];
        Ok(WebhookInfo {
            url: result["url"].as_str().unwrap_or("").into(),
            updated_at: result["updated_at"].as_i64(),
        })
    }

    /// Get bot token (for display/debug).
    pub fn token_preview(&self) -> String {
        if self.bot_token.len() > 10 {
            format!("{}...{}", &self.bot_token[..5], &self.bot_token[self.bot_token.len()-5..])
        } else {
            "***".into()
        }
    }
}
