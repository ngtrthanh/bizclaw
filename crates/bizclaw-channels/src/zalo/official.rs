//! Zalo Official Account mode — uses Zalo OA REST API.
//!
//! For business accounts via developers.zalo.me.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage};
use futures::stream::{self, Stream};

use super::client::business::ZaloBusiness;

/// Zalo OA channel — uses access token.
pub struct ZaloOfficialChannel {
    business: ZaloBusiness,
    access_token: Option<String>,
    connected: bool,
}

impl ZaloOfficialChannel {
    pub fn new() -> Self {
        Self {
            business: ZaloBusiness::new(),
            access_token: None,
            connected: false,
        }
    }

    /// Set access token from OA developer portal.
    pub fn set_access_token(&mut self, token: &str) {
        self.access_token = Some(token.to_string());
        self.connected = true;
    }
}

#[async_trait]
impl Channel for ZaloOfficialChannel {
    fn name(&self) -> &str {
        "zalo-oa"
    }

    async fn connect(&mut self) -> Result<()> {
        if self.access_token.is_none() {
            return Err(BizClawError::AuthFailed("Set access_token first".into()));
        }
        tracing::info!("Zalo OA channel connected");
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
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| BizClawError::Channel("No access token".into()))?;
        self.business
            .send_oa_message(&message.thread_id, &message.content, token)
            .await
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        Ok(Box::new(stream::pending()))
    }
}

impl Default for ZaloOfficialChannel {
    fn default() -> Self {
        Self::new()
    }
}
