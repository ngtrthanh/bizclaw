//! HTTP server implementation using Axum.

use axum::{Router, routing::get};
use axum::response::Html;
use bizclaw_core::config::GatewayConfig;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Shared state for the gateway server.
#[derive(Clone)]
pub struct AppState {
    pub config: GatewayConfig,
    pub start_time: std::time::Instant,
}

/// Serve the dashboard HTML page.
async fn dashboard_page() -> Html<&'static str> {
    Html(super::dashboard::dashboard_html())
}

/// Build the Axum router with all routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(dashboard_page))
        .route("/health", get(super::routes::health_check))
        .route("/api/v1/info", get(super::routes::system_info))
        .route("/api/v1/config", get(super::routes::get_config))
        .route("/api/v1/providers", get(super::routes::list_providers))
        .route("/api/v1/channels", get(super::routes::list_channels))
        .route("/ws", get(super::ws::ws_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

/// Start the HTTP server.
pub async fn start(config: &GatewayConfig) -> anyhow::Result<()> {
    let state = AppState {
        config: config.clone(),
        start_time: std::time::Instant::now(),
    };

    let app = build_router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Gateway server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
