//! Zalo channel module — Zalo Personal + OA.
//! Wraps the client sub-modules into the Channel trait.

pub mod client;
pub mod personal;
pub mod official;

use async_trait::async_trait;
use bizclaw_core::config::ZaloChannelConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage};
use tokio_stream::Stream;

use self::client::auth::{ZaloAuth, ZaloCredentials};
use self::client::messaging::{ZaloMessaging, ThreadType as ZaloThreadType};
use self::client::session::SessionManager;

/// Zalo channel implementation — routes to Personal or OA mode.
pub struct ZaloChannel {
    config: ZaloChannelConfig,
    auth: ZaloAuth,
    messaging: ZaloMessaging,
    session: SessionManager,
    connected: bool,
    cookie: Option<String>,
}

impl ZaloChannel {
    pub fn new(config: ZaloChannelConfig) -> Self {
        let creds = ZaloCredentials {
            imei: config.personal.imei.clone(),
            cookie: None,
            phone: None,
            user_agent: if config.personal.user_agent.is_empty() {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".into()
            } else {
                config.personal.user_agent.clone()
            },
        };
        Self {
            config,
            auth: ZaloAuth::new(creds),
            messaging: ZaloMessaging::new(),
            session: SessionManager::new(),
            connected: false,
            cookie: None,
        }
    }

    /// Login with cookie from config or parameter.
    async fn login_cookie(&mut self, cookie: &str) -> Result<()> {
        let login_data = self.auth.login_with_cookie(cookie).await?;
        self.session.set_session(
            login_data.uid.clone(),
            login_data.zpw_enk,
            login_data.zpw_key,
        ).await;
        self.cookie = Some(cookie.to_string());
        tracing::info!("Zalo logged in: uid={}", login_data.uid);
        Ok(())
    }

    /// Get QR code for login.
    pub async fn get_qr_code(&mut self) -> Result<client::auth::QrCodeResult> {
        self.auth.get_qr_code().await
    }
}

#[async_trait]
impl Channel for ZaloChannel {
    fn name(&self) -> &str { "zalo" }

    async fn connect(&mut self) -> Result<()> {
        tracing::info!("Zalo channel: connecting in {} mode...", self.config.mode);

        match self.config.mode.as_str() {
            "personal" => {
                tracing::warn!("⚠️  Zalo Personal API is unofficial. Use at your own risk.");

                // Try cookie login: from cookie_path file first, then raw cookie
                let cookie = self.try_load_cookie()?;
                if let Some(cookie) = cookie {
                    self.login_cookie(&cookie).await?;
                    self.connected = true;
                    tracing::info!("Zalo Personal: connected via cookie auth");
                } else {
                    return Err(BizClawError::AuthFailed(
                        "No Zalo cookie found. Configure cookie_path in config.toml or use QR login via admin dashboard.".into()
                    ));
                }
            }
            "official" => {
                tracing::info!("Zalo OA: connecting via official API...");
                self.connected = true;
                tracing::info!("Zalo OA: connected (official API requires Zalo OA token)");
            }
            _ => {
                return Err(BizClawError::Config(
                    format!("Unknown Zalo mode: {}", self.config.mode)
                ));
            }
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.session.invalidate().await;
        self.connected = false;
        tracing::info!("Zalo channel: disconnected");
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        // Production note: Zalo WebSocket requires zpw_enk encryption key
        // which makes true streaming complex. For now, use pending stream.
        // Messages will be processed via webhook or polling in future updates.
        tracing::info!("Zalo listener: active (webhook/polling mode)");
        Ok(Box::new(futures::stream::pending::<IncomingMessage>()))
    }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        let cookie = self.cookie.as_ref()
            .ok_or_else(|| BizClawError::Channel("Zalo not logged in".into()))?;

        self.messaging.send_text(
            &message.thread_id,
            ZaloThreadType::User,
            &message.content,
            cookie,
        ).await?;

        tracing::debug!("Zalo: message sent to {}", message.thread_id);
        Ok(())
    }

    async fn send_typing(&self, thread_id: &str) -> Result<()> {
        tracing::debug!("Zalo: typing indicator to {} (not supported by API)", thread_id);
        Ok(())
    }
}

impl ZaloChannel {
    /// Try to load cookie from cookie_path file.
    fn try_load_cookie(&self) -> Result<Option<String>> {
        let path = &self.config.personal.cookie_path;
        if path.is_empty() {
            return Ok(None);
        }

        // Expand ~ to home dir
        let expanded = if path.starts_with("~/") {
            std::env::var("HOME").ok()
                .map(|h| std::path::PathBuf::from(h).join(&path[2..]))
                .unwrap_or_else(|| std::path::PathBuf::from(path))
        } else {
            std::path::PathBuf::from(path)
        };

        if expanded.exists() {
            let content = std::fs::read_to_string(&expanded)
                .map_err(|e| BizClawError::Config(format!("Failed to read cookie file: {e}")))?;

            let trimmed = content.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }

            // Support JSON format {"cookie": "..."} or raw cookie string
            if trimmed.starts_with('{') {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if let Some(cookie) = json["cookie"].as_str() {
                        return Ok(Some(cookie.to_string()));
                    }
                }
            }

            Ok(Some(trimmed.to_string()))
        } else {
            Ok(None)
        }
    }
}
