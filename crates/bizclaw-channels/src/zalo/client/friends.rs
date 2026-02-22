//! Zalo friends — contacts management, blocking, friend requests.

use super::models::ZaloUser;
use bizclaw_core::error::{BizClawError, Result};

/// Zalo friends/contacts client.
pub struct ZaloFriends {
    client: reqwest::Client,
    base_url: String,
}

impl ZaloFriends {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://tt-chat-wpa.chat.zalo.me/api".into(),
        }
    }

    /// Get friends list.
    pub async fn get_friends(&self, cookie: &str) -> Result<Vec<ZaloUser>> {
        let response = self
            .client
            .get(format!("{}/friend/list", self.base_url))
            .header("cookie", cookie)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get friends failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid friends response: {e}")))?;

        let friends = body["data"]["friends"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        Some(ZaloUser {
                            id: f["uid"].as_str()?.into(),
                            display_name: f["displayName"].as_str()?.into(),
                            avatar: f["avatar"].as_str().map(String::from),
                            phone: f["phone"].as_str().map(String::from),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(friends)
    }

    /// Get user info by ID.
    pub async fn get_user_info(&self, user_id: &str, cookie: &str) -> Result<ZaloUser> {
        let response = self
            .client
            .get(format!("{}/friend/profile", self.base_url))
            .query(&[("fuid", user_id)])
            .header("cookie", cookie)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get user info failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid user response: {e}")))?;

        Ok(ZaloUser {
            id: body["data"]["uid"].as_str().unwrap_or(user_id).into(),
            display_name: body["data"]["displayName"]
                .as_str()
                .unwrap_or("Unknown")
                .into(),
            avatar: body["data"]["avatar"].as_str().map(String::from),
            phone: body["data"]["phone"].as_str().map(String::from),
        })
    }
}

impl Default for ZaloFriends {
    fn default() -> Self {
        Self::new()
    }
}
