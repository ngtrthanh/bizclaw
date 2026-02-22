//! Telegram Bot channel — long polling + message sending via Bot API.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage, ThreadType};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Telegram channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

fn default_true() -> bool {
    true
}
fn default_poll_interval() -> u64 {
    1
}

/// Telegram Bot channel with polling loop.
pub struct TelegramChannel {
    config: TelegramConfig,
    client: reqwest::Client,
    last_update_id: i64,
    connected: bool,
}

impl TelegramChannel {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            last_update_id: 0,
            connected: false,
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!(
            "https://api.telegram.org/bot{}/{}",
            self.config.bot_token, method
        )
    }

    /// Get updates using long polling.
    pub async fn get_updates(&mut self) -> Result<Vec<TelegramUpdate>> {
        let response = self
            .client
            .get(self.api_url("getUpdates"))
            .query(&[
                ("offset", (self.last_update_id + 1).to_string()),
                ("timeout", "30".into()),
                ("allowed_updates", "[\"message\"]".into()),
            ])
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Telegram getUpdates failed: {e}")))?;

        let body: TelegramApiResponse<Vec<TelegramUpdate>> = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid Telegram response: {e}")))?;

        if !body.ok {
            return Err(BizClawError::Channel(format!(
                "Telegram API error: {}",
                body.description.unwrap_or_default()
            )));
        }

        let updates = body.result.unwrap_or_default();
        if let Some(last) = updates.last() {
            self.last_update_id = last.update_id;
        }
        Ok(updates)
    }

    /// Send a text message.
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        });

        let response = self
            .client
            .post(self.api_url("sendMessage"))
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("sendMessage failed: {e}")))?;

        let result: TelegramApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid send response: {e}")))?;

        if !result.ok {
            return Err(BizClawError::Channel(format!(
                "Send failed: {}",
                result.description.unwrap_or_default()
            )));
        }
        Ok(())
    }

    /// Send typing indicator.
    pub async fn send_typing(&self, chat_id: i64) -> Result<()> {
        let body = serde_json::json!({
            "chat_id": chat_id,
            "action": "typing",
        });
        let _ = self
            .client
            .post(self.api_url("sendChatAction"))
            .json(&body)
            .send()
            .await;
        Ok(())
    }

    /// Get bot info.
    pub async fn get_me(&self) -> Result<TelegramUser> {
        let response = self
            .client
            .get(self.api_url("getMe"))
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("getMe failed: {e}")))?;
        let body: TelegramApiResponse<TelegramUser> = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid getMe response: {e}")))?;
        body.result
            .ok_or_else(|| BizClawError::Channel("No bot info".into()))
    }

    /// Start polling loop — returns a stream of IncomingMessages.
    pub fn start_polling(self) -> TelegramPollingStream {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Spawn polling task
        tokio::spawn(async move {
            let mut channel = self;
            tracing::info!("Telegram polling loop started");

            loop {
                match channel.get_updates().await {
                    Ok(updates) => {
                        for update in updates {
                            if let Some(msg) = update.to_incoming()
                                && tx.send(msg).is_err() {
                                    tracing::info!("Telegram polling stopped (receiver dropped)");
                                    return;
                                }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Telegram polling error: {e}");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(
                    channel.config.poll_interval,
                ))
                .await;
            }
        });

        TelegramPollingStream { rx }
    }
}

/// Stream of incoming Telegram messages from polling.
pub struct TelegramPollingStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
}

impl Stream for TelegramPollingStream {
    type Item = IncomingMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl Unpin for TelegramPollingStream {}

#[async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn connect(&mut self) -> Result<()> {
        let me = self.get_me().await?;
        tracing::info!(
            "Telegram bot: @{} ({})",
            me.username.as_deref().unwrap_or("unknown"),
            me.first_name
        );
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        let chat_id: i64 = message
            .thread_id
            .parse()
            .map_err(|_| BizClawError::Channel("Invalid chat_id".into()))?;
        self.send_message(chat_id, &message.content).await
    }

    async fn send_typing(&self, thread_id: &str) -> Result<()> {
        if let Ok(chat_id) = thread_id.parse::<i64>() {
            self.send_typing(chat_id).await?;
        }
        Ok(())
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        // For listen(), return a pending stream
        // For actual polling, use start_polling() which consumes self
        Ok(Box::new(futures::stream::pending()))
    }
}

// --- Telegram API Types ---

#[derive(Debug, Deserialize)]
pub struct TelegramApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub date: i64,
    pub reply_to_message: Option<Box<TelegramMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
}

impl TelegramUpdate {
    /// Convert to BizClaw IncomingMessage.
    pub fn to_incoming(&self) -> Option<IncomingMessage> {
        let msg = self.message.as_ref()?;
        let text = msg.text.as_ref()?;
        let from = msg.from.as_ref()?;

        // Skip bot messages
        if from.is_bot {
            return None;
        }

        Some(IncomingMessage {
            channel: "telegram".into(),
            thread_id: msg.chat.id.to_string(),
            sender_id: from.id.to_string(),
            sender_name: Some(format!(
                "{}{}",
                from.first_name,
                from.last_name
                    .as_deref()
                    .map(|l| format!(" {l}"))
                    .unwrap_or_default()
            )),
            content: text.clone(),
            thread_type: match msg.chat.chat_type.as_str() {
                "private" => ThreadType::Direct,
                _ => ThreadType::Group,
            },
            timestamp: chrono::Utc::now(),
            reply_to: msg
                .reply_to_message
                .as_ref()
                .map(|r| r.message_id.to_string()),
        })
    }
}
