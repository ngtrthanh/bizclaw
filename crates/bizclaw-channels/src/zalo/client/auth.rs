//! Zalo authentication — Cookie login, QR login, multi-account.
//! Based on zca-js (https://github.com/RFS-ADRENO/zca-js) protocol.

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
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".into(),
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

/// QR code generation result (from /authen/qr/generate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResult {
    /// QR scan code identifier
    pub code: String,
    /// Base64 PNG image data (data:image/png;base64,...)
    pub image: String,
}

/// QR login status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QrLoginStatus {
    /// QR generated, waiting for scan
    Pending { code: String, image: String },
    /// User scanned, waiting for confirm
    Scanned { avatar: String, display_name: String },
    /// User confirmed, login complete
    Confirmed,
    /// QR expired
    Expired,
    /// User declined
    Declined,
}

/// Standard headers matching zca-js browser fingerprint.
fn zalo_headers(user_agent: &str) -> Vec<(&'static str, String)> {
    vec![
        ("accept", "*/*".into()),
        ("accept-language", "vi-VN,vi;q=0.9,fr-FR;q=0.8,fr;q=0.7,en-US;q=0.6,en;q=0.5".into()),
        ("content-type", "application/x-www-form-urlencoded".into()),
        ("priority", "u=1, i".into()),
        ("sec-ch-ua", "\"Chromium\";v=\"130\", \"Google Chrome\";v=\"130\", \"Not?A_Brand\";v=\"99\"".into()),
        ("sec-ch-ua-mobile", "?0".into()),
        ("sec-ch-ua-platform", "\"Windows\"".into()),
        ("sec-fetch-dest", "empty".into()),
        ("sec-fetch-mode", "cors".into()),
        ("sec-fetch-site", "same-origin".into()),
        ("referer", "https://id.zalo.me/account?continue=https%3A%2F%2Fzalo.me%2Fpc".into()),
        ("referrer-policy", "strict-origin-when-cross-origin".into()),
        ("user-agent", user_agent.into()),
    ]
}

/// Zalo login methods.
pub struct ZaloAuth {
    credentials: ZaloCredentials,
    client: reqwest::Client,
    /// Cached login page version (e.g., "2.44.10")
    login_version: Option<String>,
}

impl ZaloAuth {
    pub fn new(credentials: ZaloCredentials) -> Self {
        Self {
            credentials,
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap_or_default(),
            login_version: None,
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

    // ─── ZCA-JS QR LOGIN FLOW ────────────────────────────

    /// Step 1: Load login page to get JS version number.
    async fn load_login_page(&mut self) -> Result<String> {
        tracing::info!("Zalo QR: loading login page...");

        let response = self.client
            .get("https://id.zalo.me/account?continue=https%3A%2F%2Fchat.zalo.me%2F")
            .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("accept-language", "vi-VN,vi;q=0.9,en-US;q=0.6,en;q=0.5")
            .header("user-agent", &self.credentials.user_agent)
            .header("referer", "https://chat.zalo.me/")
            .header("sec-fetch-dest", "document")
            .header("sec-fetch-mode", "navigate")
            .header("sec-fetch-site", "same-site")
            .header("upgrade-insecure-requests", "1")
            .send()
            .await
            .map_err(|e| BizClawError::AuthFailed(format!("Failed to load login page: {e}")))?;

        let html = response.text().await
            .map_err(|e| BizClawError::AuthFailed(format!("Failed to read login page: {e}")))?;

        // Extract version: https://stc-zlogin.zdn.vn/main-X.Y.Z.js
        let re = regex::Regex::new(r"https://stc-zlogin\.zdn\.vn/main-([\d.]+)\.js")
            .map_err(|e| BizClawError::AuthFailed(format!("Regex error: {e}")))?;

        let version = re.captures(&html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| BizClawError::AuthFailed(
                "Cannot get Zalo login version from page. API may have changed.".into()
            ))?;

        tracing::info!("Zalo QR: got login version: {}", version);
        self.login_version = Some(version.clone());
        Ok(version)
    }

    /// Step 2: Get login info (sets session cookies).
    async fn get_login_info(&self, version: &str) -> Result<()> {
        let form = format!("continue=https%3A%2F%2Fzalo.me%2Fpc&v={}", version);

        let mut req = self.client
            .post("https://id.zalo.me/account/logininfo")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        req.send().await
            .map_err(|e| BizClawError::AuthFailed(format!("logininfo failed: {e}")))?;

        Ok(())
    }

    /// Step 3: Verify client (device verification).
    async fn verify_client(&self, version: &str) -> Result<()> {
        let form = format!("type=device&continue=https%3A%2F%2Fzalo.me%2Fpc&v={}", version);

        let mut req = self.client
            .post("https://id.zalo.me/account/verify-client")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        req.send().await
            .map_err(|e| BizClawError::AuthFailed(format!("verify-client failed: {e}")))?;

        Ok(())
    }

    /// Step 4: Generate QR code. Returns QR code image (base64 PNG) and code.
    async fn generate_qr(&self, version: &str) -> Result<QrCodeResult> {
        let form = format!("continue=https%3A%2F%2Fzalo.me%2Fpc&v={}", version);

        let mut req = self.client
            .post("https://id.zalo.me/account/authen/qr/generate")
            .body(form);

        for (k, v) in zalo_headers(&self.credentials.user_agent) {
            req = req.header(k, v);
        }

        let response = req.send().await
            .map_err(|e| BizClawError::AuthFailed(format!("QR generate failed: {e}")))?;

        let text = response.text().await
            .map_err(|e| BizClawError::AuthFailed(format!("QR generate read error: {e}")))?;

        let data: serde_json::Value = serde_json::from_str(&text)
            .map_err(|_| BizClawError::AuthFailed(format!(
                "QR generate returned non-JSON. First 200 chars: {}",
                text.chars().take(200).collect::<String>()
            )))?;

        let error_code = data["error_code"].as_i64().unwrap_or(-1);
        if error_code != 0 {
            return Err(BizClawError::AuthFailed(format!(
                "QR generate error {}: {}",
                error_code,
                data["error_message"].as_str().unwrap_or("unknown")
            )));
        }

        let code = data["data"]["code"].as_str()
            .ok_or_else(|| BizClawError::AuthFailed("No 'code' in QR response".into()))?;
        let image = data["data"]["image"].as_str()
            .ok_or_else(|| BizClawError::AuthFailed("No 'image' in QR response".into()))?;

        Ok(QrCodeResult {
            code: code.to_string(),
            image: image.to_string(),
        })
    }

    /// Full QR code generation: load page → get session → verify → generate.
    /// Returns base64 PNG image and QR code identifier.
    pub async fn get_qr_code(&mut self) -> Result<QrCodeResult> {
        tracing::info!("Zalo auth: starting QR login flow (zca-js protocol)...");

        // Step 1: Load login page to get version
        let version = self.load_login_page().await?;

        // Step 2: Get login info (sets cookies)
        self.get_login_info(&version).await?;

        // Step 3: Verify client
        self.verify_client(&version).await?;

        // Step 4: Generate QR code
        let qr = self.generate_qr(&version).await?;

        tracing::info!("Zalo QR: generated successfully (code={})", qr.code);
        Ok(qr)
    }

    /// Wait for QR scan (long-polling). Returns user info when scanned.
    pub async fn wait_for_scan(&self, code: &str) -> Result<(String, String)> {
        let version = self.login_version.as_deref().unwrap_or("2.44.10");

        loop {
            let form = format!(
                "code={}&continue=https%3A%2F%2Fchat.zalo.me%2F&v={}",
                code, version
            );

            let mut req = self.client
                .post("https://id.zalo.me/account/authen/qr/waiting-scan")
                .body(form);

            for (k, v) in zalo_headers(&self.credentials.user_agent) {
                req = req.header(k, v);
            }

            let response = req.send().await
                .map_err(|e| BizClawError::AuthFailed(format!("waiting-scan failed: {e}")))?;

            let data: serde_json::Value = response.json().await
                .map_err(|e| BizClawError::AuthFailed(format!("waiting-scan parse error: {e}")))?;

            let error_code = data["error_code"].as_i64().unwrap_or(-1);

            match error_code {
                8 => continue, // Still waiting, retry
                0 => {
                    let avatar = data["data"]["avatar"].as_str().unwrap_or("").to_string();
                    let name = data["data"]["display_name"].as_str().unwrap_or("").to_string();
                    return Ok((avatar, name));
                }
                -13 => return Err(BizClawError::AuthFailed("QR code expired".into())),
                _ => return Err(BizClawError::AuthFailed(format!(
                    "waiting-scan error {}: {}",
                    error_code,
                    data["error_message"].as_str().unwrap_or("unknown")
                ))),
            }
        }
    }

    /// Wait for user confirm on phone (long-polling).
    pub async fn wait_for_confirm(&self, code: &str) -> Result<()> {
        let version = self.login_version.as_deref().unwrap_or("2.44.10");

        loop {
            let form = format!(
                "code={}&gToken=&gAction=CONFIRM_QR&continue=https%3A%2F%2Fchat.zalo.me%2F&v={}",
                code, version
            );

            let mut req = self.client
                .post("https://id.zalo.me/account/authen/qr/waiting-confirm")
                .body(form);

            for (k, v) in zalo_headers(&self.credentials.user_agent) {
                req = req.header(k, v);
            }

            let response = req.send().await
                .map_err(|e| BizClawError::AuthFailed(format!("waiting-confirm failed: {e}")))?;

            let data: serde_json::Value = response.json().await
                .map_err(|e| BizClawError::AuthFailed(format!("waiting-confirm parse error: {e}")))?;

            let error_code = data["error_code"].as_i64().unwrap_or(-1);

            match error_code {
                8 => continue,
                0 => return Ok(()),
                -13 => return Err(BizClawError::AuthFailed("User declined QR login".into())),
                _ => return Err(BizClawError::AuthFailed(format!(
                    "waiting-confirm error {}: {}",
                    error_code,
                    data["error_message"].as_str().unwrap_or("unknown")
                ))),
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
