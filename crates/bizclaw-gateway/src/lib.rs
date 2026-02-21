//! # BizClaw Gateway
//! HTTP/WebSocket gateway API with embedded web dashboard.

pub mod server;
pub mod routes;
pub mod ws;
pub mod dashboard;

use bizclaw_core::config::GatewayConfig;

/// Start the gateway HTTP server.
pub async fn start_server(config: &GatewayConfig) -> anyhow::Result<()> {
    server::start(config).await
}
