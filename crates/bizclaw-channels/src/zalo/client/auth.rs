//! Zalo authentication — Cookie login, QR login, multi-account.
//! Based on reverse-engineered Zalo Web protocol (zca-js patterns).

use serde::{Deserialize, Serialize};
use bizclaw_core::error::{BizClawError, Result};

/// Authentication credentials for Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaloCredentials {
    /// IMEI identifier (device fingerprint)
    pub imei: String,
    /// Cookie string from Zalo Web
    pub cookie: Option<String>,
    /// Phone number (for login)
    pub phone: Option<String>,
    /// User agent string
    pub user_agent: String,
}

impl Default for ZaloCredentials {
    fn default() -> Self {
        Self {
            imei: generate_imei(),
            cookie: None,
            phone: None,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".into(),
        }
    }
}

/// Login response from Zalo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub error_code: i32,
    pub error_message: String,
    pub data: Option<LoginData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub uid: String,
    pub zpw_enk: Option<String>,
    pub zpw_key: Option<String>,
    pub zpw_service_map: Option<serde_json::Value>,
}

/// Zalo login methods.
pub struct ZaloAuth {
    credentials: ZaloCredentials,
    client: reqwest::Client,
}

impl ZaloAuth {
    pub fn new(credentials: ZaloCredentials) -> Self {
        Self {
            credentials,
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap_or_default(),
        }
    }

    /// Login with cookie (fastest method).
    pub async fn login_with_cookie(&self, cookie: &str) -> Result<LoginData> {
        tracing::info!("Zalo auth: logging in with cookie...");

        // Validate cookie format
        if !cookie.contains("zpw_sek") {
            return Err(BizClawError::AuthFailed(
                "Invalid Zalo cookie: must contain zpw_sek".into()
            ));
        }

        // Send login request to Zalo Web API
        let response = self.client
            .get("https://tt-chat-wpa.chat.zalo.me/api/login/getServerInfo")
            .header("cookie", cookie)
            .header("user-agent", &self.credentials.user_agent)
            .send()
            .await
            .map_err(|e| BizClawError::AuthFailed(format!("Login request failed: {e}")))?;

        let body: serde_json::Value = response.json().await
            .map_err(|e| BizClawError::AuthFailed(format!("Invalid login response: {e}")))?;

        let error_code = body["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(BizClawError::AuthFailed(format!(
                "Login failed with error code: {} - {}",
                error_code,
                body["error_message"].as_str().unwrap_or("unknown")
            )));
        }

        Ok(LoginData {
            uid: body["data"]["uid"].as_str().unwrap_or("").into(),
            zpw_enk: body["data"]["zpw_enk"].as_str().map(String::from),
            zpw_key: body["data"]["zpw_key"].as_str().map(String::from),
            zpw_service_map: body["data"]["zpw_service_map"].as_object().map(|m| serde_json::to_value(m).unwrap_or_default()),
        })
    }

    /// Request QR code for login.
    pub async fn get_qr_code(&self) -> Result<String> {
        tracing::info!("Zalo auth: requesting QR code...");

        let body = serde_json::json!({
            "imei": self.credentials.imei,
        });

        let response = self.client
            .post("https://id.zalo.me/account/authen/qrlogin")
            .json(&body)
            .header("user-agent", &self.credentials.user_agent)
            .send()
            .await
            .map_err(|e| BizClawError::AuthFailed(format!("QR code request failed: {e}")))?;

        // Read as text first to avoid JSON parse errors on HTML responses
        let status = response.status();
        let text = response.text().await
            .map_err(|e| BizClawError::AuthFailed(format!("QR code response read error: {e}")))?;

        if !status.is_success() {
            return Err(BizClawError::AuthFailed(format!(
                "Zalo API returned status {}: {}",
                status, text.chars().take(200).collect::<String>()
            )));
        }

        // Try parsing as JSON
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(data) => {
                // Try multiple response formats
                if let Some(qr) = data["data"]["qr_code"].as_str() {
                    return Ok(qr.to_string());
                }
                if let Some(qr) = data["data"]["qrcode"].as_str() {
                    return Ok(qr.to_string());
                }
                if let Some(qr) = data["qr_code"].as_str() {
                    return Ok(qr.to_string());
                }
                // Return error with response body for debugging
                Err(BizClawError::AuthFailed(format!(
                    "No QR code field in Zalo response. Keys: {:?}",
                    data.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
                )))
            }
            Err(_) => {
                // Not JSON — likely HTML error page
                Err(BizClawError::AuthFailed(format!(
                    "Zalo trả về HTML thay vì JSON. Có thể API đã thay đổi. Vui lòng paste cookie thủ công."
                )))
            }
        }
    }

    /// Get credentials reference.
    pub fn credentials(&self) -> &ZaloCredentials {
        &self.credentials
    }
}

/// Generate a random IMEI-like device identifier.
fn generate_imei() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.r#gen::<u64>() % 999_999_999_999;
    format!("{:012}", id)
}
