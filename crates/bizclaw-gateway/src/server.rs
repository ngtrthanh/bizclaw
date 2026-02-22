//! HTTP server implementation using Axum.

use axum::{Router, Json, routing::{get, post}, extract::State};
use axum::response::Html;
use bizclaw_core::config::{GatewayConfig, BizClawConfig};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Shared state for the gateway server.
#[derive(Clone)]
pub struct AppState {
    pub gateway_config: GatewayConfig,
    pub full_config: Arc<Mutex<BizClawConfig>>,
    pub config_path: PathBuf,
    pub start_time: std::time::Instant,
    pub pairing_code: Option<String>,
}

/// Serve the dashboard HTML page.
async fn dashboard_page() -> Html<&'static str> {
    Html(super::dashboard::dashboard_html())
}

/// Pairing code auth middleware — validates X-Pairing-Code header or ?code= query.
async fn require_pairing(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // If no pairing code configured, allow all
    let Some(expected) = &state.pairing_code else {
        return next.run(req).await;
    };

    // Check header first
    let from_header = req.headers()
        .get("X-Pairing-Code")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if from_header == expected {
        return next.run(req).await;
    }

    // Check query param ?code=
    if let Some(query) = req.uri().query() {
        for pair in query.split('&') {
            if let Some(code) = pair.strip_prefix("code=") {
                if code == expected {
                    return next.run(req).await;
                }
            }
        }
    }

    axum::response::Response::builder()
        .status(axum::http::StatusCode::UNAUTHORIZED)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({"ok": false, "error": "Unauthorized — invalid or missing pairing code"}).to_string()
        ))
        .unwrap()
}

/// Verify pairing code endpoint (public).
async fn verify_pairing(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let code = body["code"].as_str().unwrap_or("");
    match &state.pairing_code {
        Some(expected) if code == expected => Json(serde_json::json!({"ok": true})),
        Some(_) => Json(serde_json::json!({"ok": false, "error": "Invalid pairing code"})),
        None => Json(serde_json::json!({"ok": true})), // no code required
    }
}

/// Build the Axum router with all routes.
pub fn build_router(state: AppState) -> Router {
    let shared = Arc::new(state);

    // Protected routes — require valid pairing code
    let protected = Router::new()
        .route("/api/v1/info", get(super::routes::system_info))
        .route("/api/v1/config", get(super::routes::get_config))
        .route("/api/v1/config/update", post(super::routes::update_config))
        .route("/api/v1/config/full", get(super::routes::get_full_config))
        .route("/api/v1/providers", get(super::routes::list_providers))
        .route("/api/v1/channels", get(super::routes::list_channels))
        .route("/api/v1/channels/update", post(super::routes::update_channel))
        .route("/api/v1/zalo/qr", post(super::routes::zalo_qr_code))
        .route("/ws", get(super::ws::ws_handler))
        .route_layer(axum::middleware::from_fn_with_state(shared.clone(), require_pairing));

    // Public routes — no auth
    let public = Router::new()
        .route("/", get(dashboard_page))
        .route("/health", get(super::routes::health_check))
        .route("/api/v1/verify-pairing", post(verify_pairing));

    protected.merge(public)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(shared)
}

/// Start the HTTP server.
pub async fn start(config: &GatewayConfig) -> anyhow::Result<()> {
    // Load full config for settings UI
    let config_path = std::env::var("BIZCLAW_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| BizClawConfig::default_path());
    let full_config = if config_path.exists() {
        BizClawConfig::load_from(&config_path).unwrap_or_default()
    } else {
        BizClawConfig::default()
    };

    let state = AppState {
        gateway_config: config.clone(),
        full_config: Arc::new(Mutex::new(full_config)),
        config_path: config_path.clone(),
        start_time: std::time::Instant::now(),
        pairing_code: if config.require_pairing {
            // Read pairing code from platform DB or generate one
            let code = std::env::var("BIZCLAW_PAIRING_CODE").ok()
                .or_else(|| {
                    // Try to extract from config directory
                    config_path.parent().and_then(|d| {
                        let pc = d.join(".pairing_code");
                        std::fs::read_to_string(pc).ok().map(|s| s.trim().to_string())
                    })
                });
            code
        } else {
            None
        },
    };

    let app = build_router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🌐 Gateway server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
