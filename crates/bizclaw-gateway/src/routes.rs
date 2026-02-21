//! API route handlers for the gateway.

use axum::{extract::State, Json};
use std::sync::Arc;

use super::server::AppState;

/// Health check endpoint.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "bizclaw-gateway",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// System information endpoint.
pub async fn system_info(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let uptime = state.start_time.elapsed();
    Json(serde_json::json!({
        "name": "BizClaw",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH),
        "uptime_secs": uptime.as_secs(),
        "gateway": {
            "host": state.config.host,
            "port": state.config.port,
            "require_pairing": state.config.require_pairing,
        }
    }))
}

/// Get current configuration (sanitized — no secrets).
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "gateway": {
            "host": state.config.host,
            "port": state.config.port,
            "require_pairing": state.config.require_pairing,
        }
    }))
}

/// List available providers.
pub async fn list_providers() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "providers": [
            {"name": "openai", "type": "cloud", "status": "available"},
            {"name": "anthropic", "type": "cloud", "status": "available"},
            {"name": "ollama", "type": "local", "status": "available"},
            {"name": "llamacpp", "type": "local", "status": "available"},
            {"name": "brain", "type": "local", "status": "available"},
        ]
    }))
}

/// List available channels.
pub async fn list_channels() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "channels": [
            {"name": "cli", "type": "interactive", "status": "available"},
            {"name": "zalo", "type": "messaging", "status": "available"},
            {"name": "telegram", "type": "messaging", "status": "planned"},
            {"name": "discord", "type": "messaging", "status": "planned"},
            {"name": "webhook", "type": "api", "status": "planned"},
        ]
    }))
}
