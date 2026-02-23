//! Zalo messaging — send/receive text, images, stickers, files.
//! Based on zca-js v2 protocol: uses dynamic service map URLs + encrypted params.

use bizclaw_core::error::{BizClawError, Result};
use serde::{Deserialize, Serialize};

/// Message types supported by Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Text,
    Image,
    Sticker,
    File,
    Link,
    Location,
    Contact,
    Gif,
    Video,
}

/// Send message request.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub thread_id: String,
    pub thread_type: ThreadType,
    pub msg_type: MessageType,
    pub content: String,
    /// Optional quote/reply to a message
    pub quote_msg_id: Option<String>,
    /// Optional mention user IDs
    pub mentions: Vec<String>,
}

/// Thread type for Zalo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ThreadType {
    /// Direct message (1:1)
    User = 0,
    /// Group chat
    Group = 1,
}

/// Zalo service map — dynamic URLs obtained after login.
/// Based on `zpw_service_map_v3` response from `getLoginInfo`.
/// Reference: https://github.com/RFS-ADRENO/zca-js/blob/main/src/context.ts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZaloServiceMap {
    /// Chat API endpoints (message sending for User threads)
    #[serde(default)]
    pub chat: Vec<String>,
    /// Group API endpoints (group management + messages)
    #[serde(default)]
    pub group: Vec<String>,
    /// Group poll endpoints (getAllGroups)
    #[serde(default)]
    pub group_poll: Vec<String>,
    /// File upload endpoints
    #[serde(default)]
    pub file: Vec<String>,
    /// Friend endpoints (friend requests)
    #[serde(default)]
    pub friend: Vec<String>,
    /// Profile endpoints (friends list, user info)
    #[serde(default)]
    pub profile: Vec<String>,
    /// Sticker endpoints
    #[serde(default)]
    pub sticker: Vec<String>,
    /// Reaction endpoints
    #[serde(default)]
    pub reaction: Vec<String>,
    /// Conversation endpoints
    #[serde(default)]
    pub conversation: Vec<String>,
}

impl ZaloServiceMap {
    /// Parse from zpw_service_map_v3 JSON value (as returned by login).
    pub fn from_login_data(map: &serde_json::Value) -> Self {
        let get_urls = |key: &str| -> Vec<String> {
            map[key]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };
        Self {
            chat: get_urls("chat"),
            group: get_urls("group"),
            group_poll: get_urls("group_poll"),
            file: get_urls("file"),
            friend: get_urls("friend"),
            profile: get_urls("profile"),
            sticker: get_urls("sticker"),
            reaction: get_urls("reaction"),
            conversation: get_urls("conversation"),
        }
    }

    /// Get the best chat URL (for User thread messaging).
    /// zca-js: api.zpwServiceMap.chat[0]
    pub fn chat_url(&self) -> &str {
        self.chat
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best group URL (for Group thread messaging + management).
    /// zca-js: api.zpwServiceMap.group[0]
    pub fn group_url(&self) -> &str {
        self.group
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best group_poll URL (for getAllGroups).
    /// zca-js: api.zpwServiceMap.group_poll[0]
    pub fn group_poll_url(&self) -> &str {
        self.group_poll
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best reaction URL.
    /// zca-js: api.zpwServiceMap.reaction[0]
    pub fn reaction_url(&self) -> &str {
        self.reaction
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best profile URL (for friends list, user info).
    /// zca-js: api.zpwServiceMap.profile[0]
    pub fn profile_url(&self) -> &str {
        self.profile
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }

    /// Get the best friend URL (for friend requests).
    /// zca-js: api.zpwServiceMap.friend[0]
    pub fn friend_url(&self) -> &str {
        self.friend
            .first()
            .map(|s| s.as_str())
            .unwrap_or("https://wpa.chat.zalo.me")
    }
}

/// Zalo messaging client — uses dynamic service map from login.
pub struct ZaloMessaging {
    client: reqwest::Client,
    /// Dynamic service map from login response
    service_map: ZaloServiceMap,
    /// Secret key from login (zpw_enk) — used for request signing
    secret_key: Option<String>,
    /// User's UID
    uid: Option<String>,
    /// API version params
    zpw_ver: u32,
    zpw_type: u32,
}

impl ZaloMessaging {
    /// Create with dynamic service map (proper zca-js v2 way).
    pub fn with_service_map(service_map: ZaloServiceMap) -> Self {
        Self {
            client: reqwest::Client::new(),
            service_map,
            secret_key: None,
            uid: None,
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Create with default endpoints (fallback).
    pub fn new() -> Self {
        Self::with_service_map(ZaloServiceMap::default())
    }

    /// Set login credentials after successful authentication.
    pub fn set_login_info(&mut self, uid: &str, secret_key: Option<&str>) {
        self.uid = Some(uid.to_string());
        self.secret_key = secret_key.map(String::from);
    }

    /// Update service map (e.g., after login).
    pub fn set_service_map(&mut self, map: ZaloServiceMap) {
        self.service_map = map;
    }

    /// Add API version query params to a URL.
    fn add_api_params(&self, base: &str) -> String {
        if base.contains('?') {
            format!(
                "{}&zpw_ver={}&zpw_type={}",
                base, self.zpw_ver, self.zpw_type
            )
        } else {
            format!(
                "{}?zpw_ver={}&zpw_type={}",
                base, self.zpw_ver, self.zpw_type
            )
        }
    }

    /// Send a text message (works for both User and Group threads).
    /// zca-js: POST {chat[0]}/api/message (User) or {group[0]}/api/group (Group)
    /// with nretry=0 query param
    pub async fn send_text(
        &self,
        thread_id: &str,
        thread_type: ThreadType,
        content: &str,
        cookie: &str,
    ) -> Result<String> {
        // zca-js: api.zpwServiceMap.chat[0] + "/api/message" (User)
        //         api.zpwServiceMap.group[0] + "/api/group" (Group)
        let base_url = if thread_type == ThreadType::User {
            format!("{}/api/message", self.service_map.chat_url())
        } else {
            format!("{}/api/group", self.service_map.group_url())
        };

        let endpoint = self.add_api_params(&format!("{}?nretry=0", base_url));

        let params = serde_json::json!({
            "toid": thread_id,
            "msg": content,
            "clientId": generate_client_id(),
        });

        let response = self
            .client
            .post(&endpoint)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Send message failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid send response: {e}")))?;

        let error_code = body["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(BizClawError::Channel(format!(
                "Send failed: {} - {}",
                error_code,
                body["error_message"].as_str().unwrap_or("unknown")
            )));
        }

        let msg_id = body["data"]["msgId"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        tracing::debug!("Sent message {} to {}", msg_id, thread_id);
        Ok(msg_id)
    }

    /// Send a reaction to a message.
    pub async fn send_reaction(
        &self,
        msg_id: &str,
        thread_id: &str,
        reaction: &str,
        cookie: &str,
    ) -> Result<()> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
            "rType": reaction,
        });

        let endpoint = self.add_api_params(&format!(
            "{}/api/message/reaction",
            self.service_map.reaction_url()
        ));

        self.client
            .post(&endpoint)
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Reaction failed: {e}")))?;

        Ok(())
    }

    /// Undo (recall) a message.
    pub async fn undo_message(&self, msg_id: &str, thread_id: &str, cookie: &str) -> Result<()> {
        let params = serde_json::json!({
            "msgId": msg_id,
            "toid": thread_id,
        });

        let endpoint =
            self.add_api_params(&format!("{}/api/message/undo", self.service_map.chat_url()));

        self.client
            .post(&endpoint)
            .header("cookie", cookie)
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Undo message failed: {e}")))?;

        Ok(())
    }

    /// Get current service map info (for debugging).
    pub fn service_info(&self) -> serde_json::Value {
        serde_json::json!({
            "chat_url": self.service_map.chat_url(),
            "group_url": self.service_map.group_url(),
            "has_secret_key": self.secret_key.is_some(),
            "uid": self.uid.as_deref().unwrap_or("not set"),
            "zpw_ver": self.zpw_ver,
        })
    }
}

impl Default for ZaloMessaging {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a client-side message ID.
fn generate_client_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.r#gen::<u64>() % 9_999_999_999;
    format!("cli_{}", id)
}
