//! Discord Bot channel — REST API + Gateway WebSocket.
//!
//! Connects to Discord Gateway for real-time events (messages, reactions, etc.)
//! and uses REST API for sending messages.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage, ThreadType};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

/// Discord channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Gateway intents bitmask.
    #[serde(default = "default_intents")]
    pub intents: u64,
}

fn default_true() -> bool {
    true
}
fn default_intents() -> u64 {
    // GUILDS | GUILD_MESSAGES | DIRECT_MESSAGES | MESSAGE_CONTENT
    (1 << 0) | (1 << 9) | (1 << 12) | (1 << 15)
}

/// Discord Bot channel.
pub struct DiscordChannel {
    config: DiscordConfig,
    client: reqwest::Client,
    connected: bool,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    "Authorization",
                    format!("Bot {}", config.bot_token).parse().unwrap(),
                );
                h.insert("User-Agent", "BizClaw/0.1".parse().unwrap());
                h
            })
            .build()
            .unwrap_or_default();

        Self {
            config,
            client,
            connected: false,
        }
    }

    /// Send a message to a channel.
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");
        let body = serde_json::json!({ "content": content });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Discord send failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(BizClawError::Channel(format!("Discord {status}: {text}")));
        }
        Ok(())
    }

    /// Send typing indicator.
    pub async fn send_typing_indicator(&self, channel_id: &str) -> Result<()> {
        let url = format!("https://discord.com/api/v10/channels/{channel_id}/typing");
        let _ = self.client.post(&url).send().await;
        Ok(())
    }

    /// Get current bot info.
    pub async fn get_me(&self) -> Result<DiscordUser> {
        let response = self
            .client
            .get("https://discord.com/api/v10/users/@me")
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("getMe failed: {e}")))?;
        response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid response: {e}")))
    }

    /// Get Gateway WebSocket URL.
    pub async fn get_gateway_url(&self) -> Result<String> {
        let response = self
            .client
            .get("https://discord.com/api/v10/gateway/bot")
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Gateway request failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid gateway response: {e}")))?;

        body["url"]
            .as_str()
            .map(|s| format!("{s}/?v=10&encoding=json"))
            .ok_or_else(|| BizClawError::Channel("No gateway URL".into()))
    }

    /// Start Gateway WebSocket connection — returns a stream of IncomingMessages.
    /// Auto-reconnects on disconnect with exponential backoff.
    pub fn start_gateway(self) -> DiscordGatewayStream {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let channel = self;
            let mut backoff_secs: u64 = 5;

            // ═══ Reconnect loop ═══
            loop {
                tracing::info!("Discord Gateway connecting...");

                // Get gateway URL
                let gateway_url = match channel.get_gateway_url().await {
                    Ok(url) => url,
                    Err(e) => {
                        tracing::error!(
                            "Failed to get gateway URL: {e}, retrying in {backoff_secs}s..."
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                        backoff_secs = (backoff_secs * 2).min(60);
                        continue;
                    }
                };

                // Connect WebSocket
                let ws_result = tokio_tungstenite::connect_async(&gateway_url).await;
                let (mut ws, _) = match ws_result {
                    Ok(conn) => conn,
                    Err(e) => {
                        tracing::error!(
                            "Gateway WebSocket failed: {e}, retrying in {backoff_secs}s..."
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                        backoff_secs = (backoff_secs * 2).min(60);
                        continue;
                    }
                };

                // Reset backoff on successful connect
                backoff_secs = 5;
                tracing::info!("Discord Gateway connected");

                use futures::{SinkExt, StreamExt};
                use tokio_tungstenite::tungstenite::Message as WsMsg;

                let mut heartbeat_interval_ms: u64 = 41250;
                let mut seq: Option<u64> = None;
                let mut identified = false;

                loop {
                    tokio::select! {
                        msg = ws.next() => {
                            match msg {
                                Some(Ok(WsMsg::Text(text))) => {
                                    let payload: serde_json::Value = match serde_json::from_str(&text) {
                                        Ok(v) => v,
                                        Err(_) => continue,
                                    };

                                    let op = payload["op"].as_u64().unwrap_or(0);
                                    if let Some(s) = payload["s"].as_u64() {
                                        seq = Some(s);
                                    }

                                    match op {
                                        10 => {
                                            heartbeat_interval_ms = payload["d"]["heartbeat_interval"]
                                                .as_u64().unwrap_or(41250);
                                            tracing::debug!("Gateway Hello: heartbeat={}ms", heartbeat_interval_ms);

                                            if !identified {
                                                let identify = serde_json::json!({
                                                    "op": 2,
                                                    "d": {
                                                        "token": channel.config.bot_token,
                                                        "intents": channel.config.intents,
                                                        "properties": {
                                                            "os": std::env::consts::OS,
                                                            "browser": "bizclaw",
                                                            "device": "bizclaw"
                                                        }
                                                    }
                                                });
                                                let _ = ws.send(WsMsg::Text(identify.to_string())).await;
                                                identified = true;
                                            }
                                        }
                                        11 => { tracing::trace!("Heartbeat ACK"); }
                                        0 => {
                                            let event_name = payload["t"].as_str().unwrap_or("");
                                            match event_name {
                                                "READY" => {
                                                    let user = payload["d"]["user"]["username"]
                                                        .as_str().unwrap_or("unknown");
                                                    tracing::info!("Discord Gateway READY as {user}");
                                                }
                                                "MESSAGE_CREATE" => {
                                                    let d = &payload["d"];
                                                    if d["author"]["bot"].as_bool().unwrap_or(false) {
                                                        continue;
                                                    }

                                                    let msg = IncomingMessage {
                                                        channel: "discord".into(),
                                                        thread_id: d["channel_id"].as_str()
                                                            .unwrap_or("").into(),
                                                        sender_id: d["author"]["id"].as_str()
                                                            .unwrap_or("").into(),
                                                        sender_name: d["author"]["username"].as_str()
                                                            .map(String::from),
                                                        content: d["content"].as_str()
                                                            .unwrap_or("").into(),
                                                        thread_type: if d["guild_id"].is_null() {
                                                            ThreadType::Direct
                                                        } else {
                                                            ThreadType::Group
                                                        },
                                                        timestamp: chrono::Utc::now(),
                                                        reply_to: d["referenced_message"]["id"]
                                                            .as_str().map(String::from),
                                                    };

                                                    if tx.send(msg).is_err() {
                                                        tracing::info!("Discord stream closed (receiver dropped)");
                                                        return; // Stop completely
                                                    }
                                                }
                                                _ => { tracing::trace!("Ignoring event: {event_name}"); }
                                            }
                                        }
                                        7 => {
                                            tracing::warn!("Gateway requesting reconnect");
                                            break; // → outer reconnect loop
                                        }
                                        9 => {
                                            tracing::warn!("Invalid session, re-identifying");
                                            identified = false;
                                        }
                                        _ => {}
                                    }
                                }
                                Some(Ok(WsMsg::Close(_))) => {
                                    tracing::warn!("Discord Gateway closed by server");
                                    break; // → reconnect
                                }
                                Some(Err(e)) => {
                                    tracing::error!("Gateway error: {e}");
                                    break; // → reconnect
                                }
                                None => break,
                                _ => {}
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(heartbeat_interval_ms)) => {
                            let heartbeat = serde_json::json!({
                                "op": 1,
                                "d": seq,
                            });
                            if ws.send(WsMsg::Text(heartbeat.to_string())).await.is_err() {
                                tracing::error!("Heartbeat send failed");
                                break; // → reconnect
                            }
                            tracing::trace!("Heartbeat sent (seq={:?})", seq);
                        }
                    }
                }

                // Disconnected — reconnect after backoff
                tracing::info!("Discord Gateway disconnected, reconnecting in {backoff_secs}s...");
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(60);
            } // end reconnect loop
        });

        DiscordGatewayStream { rx }
    }
}

/// Stream of incoming Discord messages from Gateway.
pub struct DiscordGatewayStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
}

impl Stream for DiscordGatewayStream {
    type Item = IncomingMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl Unpin for DiscordGatewayStream {}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    async fn connect(&mut self) -> Result<()> {
        let me = self.get_me().await?;
        tracing::info!("Discord bot: {} ({})", me.username, me.id);
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
        self.send_message(&message.thread_id, &message.content)
            .await
    }

    async fn send_typing(&self, thread_id: &str) -> Result<()> {
        self.send_typing_indicator(thread_id).await
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        Ok(Box::new(futures::stream::pending()))
    }
}

// --- Discord API Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: Option<String>,
    pub bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
    pub author: DiscordUser,
    pub content: String,
    pub guild_id: Option<String>,
}
