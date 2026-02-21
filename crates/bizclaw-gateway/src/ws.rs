//! WebSocket handler for real-time chat via gateway.

use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use std::sync::Arc;
use super::server::AppState;

/// WebSocket upgrade handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle a WebSocket connection.
async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("WebSocket client connected");

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "BizClaw Gateway — WebSocket connected",
        "version": env!("CARGO_PKG_VERSION"),
    });
    if let Err(e) = socket.send(Message::Text(welcome.to_string().into())).await {
        tracing::error!("Failed to send welcome: {e}");
        return;
    }

    // Message loop
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("WS received: {}", text);

                // Parse incoming message
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        let msg_type = json["type"].as_str().unwrap_or("unknown");

                        match msg_type {
                            "chat" => {
                                let content = json["content"].as_str().unwrap_or("");
                                let provider = json["provider"].as_str().unwrap_or("default");

                                tracing::debug!("Chat request: provider={}, content_len={}", provider, content.len());

                                let response = serde_json::json!({
                                    "type": "chat_response",
                                    "content": format!("[gateway] Received: {}", content),
                                    "provider": provider,
                                    "status": "stub",
                                });

                                if let Err(e) = socket.send(Message::Text(response.to_string().into())).await {
                                    tracing::error!("Failed to send response: {e}");
                                    break;
                                }
                            }
                            "ping" => {
                                let pong = serde_json::json!({
                                    "type": "pong",
                                    "timestamp": chrono::Utc::now().timestamp(),
                                });
                                let _ = socket.send(Message::Text(pong.to_string().into())).await;
                            }
                            _ => {
                                let error = serde_json::json!({
                                    "type": "error",
                                    "message": format!("Unknown message type: {msg_type}"),
                                });
                                let _ = socket.send(Message::Text(error.to_string().into())).await;
                            }
                        }
                    }
                    Err(e) => {
                        let error = serde_json::json!({
                            "type": "error",
                            "message": format!("Invalid JSON: {e}"),
                        });
                        let _ = socket.send(Message::Text(error.to_string().into())).await;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                let _ = socket.send(Message::Pong(data)).await;
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket client disconnected");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {e}");
                break;
            }
            _ => {}
        }
    }

    tracing::info!("WebSocket connection closed");
}
