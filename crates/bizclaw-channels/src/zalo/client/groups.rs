//! Zalo group management — create, add/remove members, settings.

use super::models::ZaloGroup;
use bizclaw_core::error::{BizClawError, Result};

/// Zalo groups client.
pub struct ZaloGroups {
    client: reqwest::Client,
    base_url: String,
}

impl ZaloGroups {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://tt-chat-wpa.chat.zalo.me/api".into(),
        }
    }

    /// Get groups list.
    pub async fn get_groups(&self, cookie: &str) -> Result<Vec<ZaloGroup>> {
        let response = self
            .client
            .get(format!("{}/group/list", self.base_url))
            .header("cookie", cookie)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get groups failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid groups response: {e}")))?;

        let groups = body["data"]["groups"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|g| {
                        Some(ZaloGroup {
                            id: g["groupId"].as_str()?.into(),
                            name: g["name"].as_str().unwrap_or("").into(),
                            member_count: g["totalMember"].as_u64().unwrap_or(0) as u32,
                            avatar: g["avt"].as_str().map(String::from),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(groups)
    }

    /// Get group info.
    pub async fn get_group_info(&self, group_id: &str, cookie: &str) -> Result<ZaloGroup> {
        let response = self
            .client
            .get(format!("{}/group/info", self.base_url))
            .query(&[("groupId", group_id)])
            .header("cookie", cookie)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get group info failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid group response: {e}")))?;

        Ok(ZaloGroup {
            id: group_id.into(),
            name: body["data"]["name"].as_str().unwrap_or("").into(),
            member_count: body["data"]["totalMember"].as_u64().unwrap_or(0) as u32,
            avatar: body["data"]["avt"].as_str().map(String::from),
        })
    }
}

impl Default for ZaloGroups {
    fn default() -> Self {
        Self::new()
    }
}
