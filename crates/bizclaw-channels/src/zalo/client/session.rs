//! Zalo session management — cookie jar, keep-alive, reconnection.

use std::sync::Arc;
use tokio::sync::RwLock;

/// Zalo session state.
#[derive(Debug, Clone)]
pub struct ZaloSession {
    /// User ID
    pub uid: String,
    /// Encrypted key for WebSocket
    pub zpw_enk: Option<String>,
    /// Service key
    pub zpw_key: Option<String>,
    /// WebSocket URL
    pub ws_url: Option<String>,
    /// Session active flag
    pub active: bool,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
}

impl Default for ZaloSession {
    fn default() -> Self {
        Self {
            uid: String::new(),
            zpw_enk: None,
            zpw_key: None,
            ws_url: None,
            active: false,
            last_heartbeat: 0,
        }
    }
}

/// Thread-safe session manager.
pub struct SessionManager {
    session: Arc<RwLock<ZaloSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(ZaloSession::default())),
        }
    }

    /// Update session after login.
    pub async fn set_session(&self, uid: String, zpw_enk: Option<String>, zpw_key: Option<String>) {
        let mut session = self.session.write().await;
        session.uid = uid;
        session.zpw_enk = zpw_enk;
        session.zpw_key = zpw_key;
        session.active = true;
        session.last_heartbeat = current_timestamp();
    }

    /// Check if session is active.
    pub async fn is_active(&self) -> bool {
        let session = self.session.read().await;
        session.active
    }

    /// Get current user ID.
    pub async fn uid(&self) -> String {
        self.session.read().await.uid.clone()
    }

    /// Update heartbeat timestamp.
    pub async fn heartbeat(&self) {
        let mut session = self.session.write().await;
        session.last_heartbeat = current_timestamp();
    }

    /// Invalidate session.
    pub async fn invalidate(&self) {
        let mut session = self.session.write().await;
        session.active = false;
    }

    /// Get session clone.
    pub async fn get_session(&self) -> ZaloSession {
        self.session.read().await.clone()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
