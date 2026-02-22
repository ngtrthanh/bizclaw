//! Zalo business features — catalog, products, auto-reply.
//! For Zalo OA (Official Account) mode.

use super::models::{ZaloCatalog, ZaloProduct};
use bizclaw_core::error::{BizClawError, Result};

/// Zalo business/OA client.
pub struct ZaloBusiness {
    client: reqwest::Client,
    base_url: String,
}

impl ZaloBusiness {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://openapi.zalo.me/v3.0/oa".into(),
        }
    }

    /// Get product catalog (OA mode).
    pub async fn get_catalog(&self, access_token: &str) -> Result<Vec<ZaloCatalog>> {
        let response = self
            .client
            .get(format!("{}/store/getslice", self.base_url))
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Get catalog failed: {e}")))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("Invalid catalog response: {e}")))?;

        // Parse catalog items
        let catalogs = body["data"]["products"]
            .as_array()
            .map(|arr| {
                vec![ZaloCatalog {
                    id: "default".into(),
                    name: "Product Catalog".into(),
                    products: arr
                        .iter()
                        .filter_map(|p| {
                            Some(ZaloProduct {
                                id: p["id"].as_str()?.into(),
                                name: p["name"].as_str().unwrap_or("").into(),
                                price: p["price"].as_f64(),
                                photo_url: p["photo"].as_str().map(String::from),
                            })
                        })
                        .collect(),
                }]
            })
            .unwrap_or_default();

        Ok(catalogs)
    }

    /// Send OA message (requires access token).
    pub async fn send_oa_message(
        &self,
        user_id: &str,
        message: &str,
        access_token: &str,
    ) -> Result<()> {
        let body = serde_json::json!({
            "recipient": {
                "user_id": user_id
            },
            "message": {
                "text": message
            }
        });

        let response = self
            .client
            .post(format!("{}/message/cs", self.base_url))
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("OA send failed: {e}")))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| BizClawError::Channel(format!("OA send response error: {e}")))?;

        if result["error"].as_i64().unwrap_or(-1) != 0 {
            return Err(BizClawError::Channel(format!(
                "OA send error: {}",
                result["message"].as_str().unwrap_or("unknown")
            )));
        }

        Ok(())
    }
}

impl Default for ZaloBusiness {
    fn default() -> Self {
        Self::new()
    }
}
