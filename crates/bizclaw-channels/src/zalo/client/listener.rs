//! Zalo WebSocket event listener.
//! Handles: message, reaction, undo, group_event, typing.

use futures::StreamExt;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use bizclaw_core::error::{BizClawError, Result};
use super::models::ZaloMessage;

/// WebSocket event types from Zalo.
#[derive(Debug, Clone)]
pub enum ZaloEvent {
    /// New message received
    Message(ZaloMessage),
    /// Message recalled/undone
    MessageUndo { msg_id: String, thread_id: String },
    /// Reaction on a message
    Reaction { msg_id: String, reactor_id: String, reaction: String },
    /// Typing indicator
    Typing { thread_id: String, user_id: String },
    /// Group member event (join/leave/kicked)
    GroupEvent { group_id: String, event_type: String, data: serde_json::Value },
    /// Connection state changed
    ConnectionState(ConnectionState),
    /// Raw/unknown event
    Raw(serde_json::Value),
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
}

/// Zalo WebSocket listener.
pub struct ZaloListener {
    ws_url: String,
    connected: bool,
}

impl ZaloListener {
    pub fn new(ws_url: &str) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            connected: false,
        }
    }

    /// Connect to Zalo WebSocket server.
    pub async fn connect(&mut self) -> Result<()> {
        tracing::info!("Connecting to Zalo WebSocket: {}", self.ws_url);

        let (ws_stream, _response) = tokio_tungstenite::connect_async(&self.ws_url)
            .await
            .map_err(|e| BizClawError::Channel(format!("WebSocket connect failed: {e}")))?;

        self.connected = true;
        tracing::info!("Zalo WebSocket connected");

        // Split the stream for reading and writing
        let (_write, mut read) = ws_stream.split();

        // Process incoming messages
        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    match self.parse_event(&text) {
                        Ok(event) => {
                            tracing::debug!("Zalo event: {:?}", event);
                            // Note: Events are logged. Integration with ZaloChannel
                            // message stream requires mpsc sender injection at construction time.
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse Zalo event: {e}");
                        }
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    tracing::trace!("Zalo ping received ({} bytes)", data.len());
                }
                Ok(WsMessage::Close(frame)) => {
                    tracing::info!("Zalo WebSocket closed: {:?}", frame);
                    self.connected = false;
                    break;
                }
                Err(e) => {
                    tracing::error!("Zalo WebSocket error: {e}");
                    self.connected = false;
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Parse a WebSocket text message into a ZaloEvent.
    fn parse_event(&self, text: &str) -> Result<ZaloEvent> {
        let json: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| BizClawError::Channel(format!("Invalid JSON: {e}")))?;

        let cmd = json["cmd"].as_i64().unwrap_or(0);

        match cmd {
            501 => {
                // New message
                Ok(ZaloEvent::Message(ZaloMessage {
                    msg_id: json["data"]["msgId"].as_str().unwrap_or("").into(),
                    thread_id: json["data"]["toid"].as_str().unwrap_or("").into(),
                    sender_id: json["data"]["uidFrom"].as_str().unwrap_or("").into(),
                    content: super::models::ZaloMessageContent::Text(
                        json["data"]["content"].as_str().unwrap_or("").into()
                    ),
                    timestamp: json["data"]["ts"].as_u64().unwrap_or(0),
                    is_self: false,
                }))
            }
            521 => {
                // Message undo
                Ok(ZaloEvent::MessageUndo {
                    msg_id: json["data"]["msgId"].as_str().unwrap_or("").into(),
                    thread_id: json["data"]["toid"].as_str().unwrap_or("").into(),
                })
            }
            612 => {
                // Reaction
                Ok(ZaloEvent::Reaction {
                    msg_id: json["data"]["msgId"].as_str().unwrap_or("").into(),
                    reactor_id: json["data"]["uidFrom"].as_str().unwrap_or("").into(),
                    reaction: json["data"]["rType"].as_str().unwrap_or("").into(),
                })
            }
            _ => {
                Ok(ZaloEvent::Raw(json))
            }
        }
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
