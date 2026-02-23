//! Zalo friends — contacts management, friend requests.
//! Based on zca-js v2 protocol: uses dynamic service map URLs from login.
//! Reference: https://github.com/RFS-ADRENO/zca-js/blob/main/src/apis/getAllFriends.ts

use bizclaw_core::error::{BizClawError, Result};
use super::models::ZaloUser;

/// Zalo friends/contacts client.
/// Uses dynamic service map URL from `zpw_service_map_v3.profile[0]`.
pub struct ZaloFriends {
    client: reqwest::Client,
    /// Dynamic profile service URL from login (e.g., "https://profile-wpa.chat.zalo.me")
    profile_url: String,
    /// Dynamic friend service URL from login
    friend_url: String,
    /// API version params
    zpw_ver: u32,
    zpw_type: u32,
}

impl ZaloFriends {
    /// Create with dynamic service map URLs (proper zca-js way).
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            profile_url: String::new(),
            friend_url: String::new(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Create with URLs from service map.
    pub fn with_urls(profile_url: &str, friend_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            profile_url: profile_url.to_string(),
            friend_url: friend_url.to_string(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Create with legacy URL (for backward compat).
    pub fn with_url(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            profile_url: url.to_string(),
            friend_url: url.to_string(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Set profile service URL (from zpw_service_map_v3.profile[0]).
    pub fn set_profile_url(&mut self, url: &str) {
        self.profile_url = url.to_string();
    }

    /// Set friend service URL (from zpw_service_map_v3.friend[0]).
    pub fn set_friend_url(&mut self, url: &str) {
        self.friend_url = url.to_string();
    }

    /// Build URL with API version params.
    fn make_url(&self, base: &str) -> String {
        if base.contains('?') {
            format!("{}&zpw_ver={}&zpw_type={}", base, self.zpw_ver, self.zpw_type)
        } else {
            format!("{}?zpw_ver={}&zpw_type={}", base, self.zpw_ver, self.zpw_type)
        }
    }

    /// Get friends list.
    /// zca-js: GET {profile[0]}/api/social/friend/getfriends?params={encrypted}
    pub async fn get_friends(&self, cookie: &str) -> Result<Vec<ZaloUser>> {
        if self.profile_url.is_empty() {
            return Err(BizClawError::Channel(
                "Profile service URL not set. Login first to get zpw_service_map_v3.".into()
            ));
        }

        let url = self.make_url(&format!(
            "{}/api/social/friend/getfriends",
            self.profile_url
        ));

        let response = self.client
            .get(&url)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get friends failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid friends response: {e}")))?;

        let friends = body["data"]
            .as_array()
            .map(|arr| {
                arr.iter().filter_map(|f| {
                    Some(ZaloUser {
                        id: f["userId"].as_str()
                            .or_else(|| f["uid"].as_str())?.into(),
                        display_name: f["displayName"].as_str()
                            .or_else(|| f["zaloName"].as_str())
                            .unwrap_or("").into(),
                        avatar: f["avatar"].as_str().map(String::from),
                        phone: f["phoneNumber"].as_str()
                            .or_else(|| f["phone"].as_str())
                            .map(String::from),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(friends)
    }

    /// Get user info by ID.
    /// zca-js: getUserInfo via profile service
    pub async fn get_user_info(&self, user_id: &str, cookie: &str) -> Result<ZaloUser> {
        if self.profile_url.is_empty() {
            return Err(BizClawError::Channel(
                "Profile service URL not set. Login first to get zpw_service_map_v3.".into()
            ));
        }

        let url = self.make_url(&format!(
            "{}/api/social/friend/getprofile",
            self.profile_url
        ));

        let response = self.client
            .get(&url)
            .query(&[("fuid", user_id)])
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get user info failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid user response: {e}")))?;

        Ok(ZaloUser {
            id: body["data"]["userId"].as_str()
                .or_else(|| body["data"]["uid"].as_str())
                .unwrap_or(user_id).into(),
            display_name: body["data"]["displayName"].as_str()
                .or_else(|| body["data"]["zaloName"].as_str())
                .unwrap_or("Unknown").into(),
            avatar: body["data"]["avatar"].as_str().map(String::from),
            phone: body["data"]["phoneNumber"].as_str()
                .or_else(|| body["data"]["phone"].as_str())
                .map(String::from),
        })
    }

    /// Send friend request.
    /// zca-js: POST {friend[0]}/api/friend/sendreq
    pub async fn send_friend_request(
        &self,
        user_id: &str,
        message: &str,
        cookie: &str,
    ) -> Result<()> {
        if self.friend_url.is_empty() {
            return Err(BizClawError::Channel(
                "Friend service URL not set. Login first.".into()
            ));
        }

        let url = self.make_url(&format!(
            "{}/api/friend/sendreq",
            self.friend_url
        ));

        let params = serde_json::json!({
            "toid": user_id,
            "msg": message,
            "reqsrc": 30,
        });

        self.client
            .post(&url)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Send friend request failed: {e}")))?;

        Ok(())
    }
}

impl Default for ZaloFriends {
    fn default() -> Self { Self::new() }
}
