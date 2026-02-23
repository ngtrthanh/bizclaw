//! Zalo group management — get groups, group info, members.
//! Based on zca-js v2 protocol: uses dynamic service map URLs from login.
//! Reference: https://github.com/RFS-ADRENO/zca-js/blob/main/src/apis/getAllGroups.ts

use bizclaw_core::error::{BizClawError, Result};
use super::models::ZaloGroup;

/// Zalo groups client.
/// Uses dynamic service map URLs from `zpw_service_map_v3`.
pub struct ZaloGroups {
    client: reqwest::Client,
    /// Dynamic group service URL (zpw_service_map_v3.group[0])
    group_url: String,
    /// Dynamic group_poll service URL (zpw_service_map_v3.group_poll[0])
    group_poll_url: String,
    /// API version params
    zpw_ver: u32,
    zpw_type: u32,
}

impl ZaloGroups {
    /// Create with empty URLs (must set after login).
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            group_url: String::new(),
            group_poll_url: String::new(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Create with URLs from service map.
    pub fn with_urls(group_url: &str, group_poll_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            group_url: group_url.to_string(),
            group_poll_url: group_poll_url.to_string(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Create with legacy URL (for backward compat).
    pub fn with_url(url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            group_url: url.to_string(),
            group_poll_url: url.to_string(),
            zpw_ver: 671,
            zpw_type: 30,
        }
    }

    /// Set group service URL (from zpw_service_map_v3.group[0]).
    pub fn set_group_url(&mut self, url: &str) {
        self.group_url = url.to_string();
    }

    /// Set group_poll service URL (from zpw_service_map_v3.group_poll[0]).
    pub fn set_group_poll_url(&mut self, url: &str) {
        self.group_poll_url = url.to_string();
    }

    /// Build URL with API version params.
    fn make_url(&self, base: &str) -> String {
        if base.contains('?') {
            format!("{}&zpw_ver={}&zpw_type={}", base, self.zpw_ver, self.zpw_type)
        } else {
            format!("{}?zpw_ver={}&zpw_type={}", base, self.zpw_ver, self.zpw_type)
        }
    }

    /// Get all groups (list group IDs + versions).
    /// zca-js: GET {group_poll[0]}/api/group/getlg/v4
    pub async fn get_groups(&self, cookie: &str) -> Result<Vec<ZaloGroup>> {
        if self.group_poll_url.is_empty() {
            return Err(BizClawError::Channel(
                "Group poll service URL not set. Login first to get zpw_service_map_v3.".into()
            ));
        }

        let url = self.make_url(&format!(
            "{}/api/group/getlg/v4",
            self.group_poll_url
        ));

        let response = self.client
            .get(&url)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get groups failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid groups response: {e}")))?;

        // zca-js response: { version, gridVerMap: { groupId: version } }
        let groups = body["data"]["gridVerMap"]
            .as_object()
            .map(|map| {
                map.keys().map(|group_id| {
                    ZaloGroup {
                        id: group_id.clone(),
                        name: String::new(), // Need getGroupInfo for details
                        member_count: 0,
                        avatar: None,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(groups)
    }

    /// Get detailed group info.
    /// zca-js: POST {group[0]}/api/group/getmg-v2
    pub async fn get_group_info(&self, group_id: &str, cookie: &str) -> Result<ZaloGroup> {
        if self.group_url.is_empty() {
            return Err(BizClawError::Channel(
                "Group service URL not set. Login first to get zpw_service_map_v3.".into()
            ));
        }

        let url = self.make_url(&format!(
            "{}/api/group/getmg-v2",
            self.group_url
        ));

        // zca-js sends gridVerMap as encrypted params
        let grid_ver_map = serde_json::json!({ group_id: 0 });
        let params = serde_json::json!({
            "gridVerMap": grid_ver_map.to_string(),
        });

        let response = self.client
            .post(&url)
            .header("cookie", cookie)
            .header("origin", "https://chat.zalo.me")
            .header("referer", "https://chat.zalo.me/")
            .form(&params)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get group info failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid group response: {e}")))?;

        // Response: { gridInfoMap: { groupId: { name, totalMember, ... } } }
        let info = &body["data"]["gridInfoMap"][group_id];

        Ok(ZaloGroup {
            id: group_id.into(),
            name: info["name"].as_str().unwrap_or("").into(),
            member_count: info["totalMember"].as_u64().unwrap_or(0) as u32,
            avatar: info["avt"].as_str().map(String::from),
        })
    }
}

impl Default for ZaloGroups {
    fn default() -> Self { Self::new() }
}
