use crate::admin::AdminState;
use axum::{Extension, Json, extract::State};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RegisterReq {
    pub email: String,
    pub password: String,
    pub company_name: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordReq {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordReq {
    pub token: String,
    pub new_password: String,
}

/// Minimum password length — unified across all endpoints
const MIN_PASSWORD_LENGTH: usize = 8;

/// Validate email format (stricter than just `contains('@')`)
fn is_valid_email(email: &str) -> bool {
    // Must contain exactly one @, at least one char before @, domain with dot
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 { return false; }
    let local = parts[0];
    let domain = parts[1];
    // Local part: non-empty, no spaces
    if local.is_empty() || local.contains(' ') { return false; }
    // Domain: must have at least one dot, no spaces, min 3 chars (a.b)
    if domain.len() < 3 || !domain.contains('.') || domain.contains(' ') { return false; }
    // Domain must not start/end with dot or hyphen
    if domain.starts_with('.') || domain.ends_with('.') 
       || domain.starts_with('-') || domain.ends_with('-') { return false; }
    // TLD must be at least 2 chars
    if let Some(tld) = domain.rsplit('.').next() {
        if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphanumeric()) { return false; }
    }
    // Total length check
    email.len() >= 5 && email.len() <= 254
}

/// Sanitize internal error messages — never expose SQL/file paths to clients
fn sanitize_error(internal_msg: &str) -> String {
    // Log the real error server-side
    tracing::error!("[security] Internal error: {}", internal_msg);
    // Return generic message to client
    "An internal error occurred. Please try again or contact support.".to_string()
}

pub fn generate_safe_slug(company_name: &str) -> String {
    // Only keep ASCII alphanumeric + spaces, skip Unicode diacritics
    let mut slug: String = company_name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-");
    
    // Collapse multiple hyphens and trim
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug = slug.trim_matches('-').to_string();
        
    let blacklist = ["dev", "admin", "app", "apps", "www", "test", "staging", "api", "smtp", "mail", "ftp", "ns", "cdn", "local", "root", "sys", "system"];
    
    if blacklist.contains(&slug.as_str()) || slug.is_empty() {
        slug = format!("tenant-{}", uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    }
    
    slug
}

pub async fn register_handler(
    State(state): State<Arc<AdminState>>,
    Json(req): Json<RegisterReq>,
) -> Json<serde_json::Value> {
    // Rate limiting — max 3 registration attempts per email per 10 minutes
    {
        let mut attempts = state.register_attempts.lock().unwrap();
        let now = std::time::Instant::now();
        if let Some((count, first_at)) = attempts.get(&req.email) {
            if now.duration_since(*first_at).as_secs() < 600 && *count >= 3 {
                return Json(serde_json::json!({
                    "ok": false,
                    "error": "Quá nhiều lần đăng ký. Vui lòng thử lại sau 10 phút."
                }));
            }
            if now.duration_since(*first_at).as_secs() >= 600 {
                attempts.remove(&req.email);
            }
        }
        let entry = attempts.entry(req.email.clone()).or_insert((0, now));
        entry.0 += 1;
    }

    if req.email.is_empty() || req.password.is_empty() || req.company_name.is_empty() {
        return Json(serde_json::json!({"ok": false, "error": "Email, password, and company name are required"}));
    }
    // Email format validation (stricter)
    if !is_valid_email(&req.email) {
        return Json(serde_json::json!({"ok": false, "error": "Email không hợp lệ"}));
    }
    // Password strength validation (unified constant)
    if req.password.len() < MIN_PASSWORD_LENGTH {
        return Json(serde_json::json!({"ok": false, "error": format!("Mật khẩu phải có ít nhất {} ký tự", MIN_PASSWORD_LENGTH)}));
    }

    let password = req.password.clone();
    let hash = match tokio::task::spawn_blocking(move || crate::auth::hash_password(&password)).await.unwrap_or_else(|e| Err(e.to_string())) {
        Ok(h) => h,
        Err(e) => return Json(serde_json::json!({"ok": false, "error": sanitize_error(&format!("Hash error: {e}"))})),
    };

    let base_slug = generate_safe_slug(&req.company_name);
    let mut final_slug = base_slug.clone();
    
    // Find unique slug
    {
        let db = state.db.lock().unwrap();
        let mut counter = 1;
        while db.is_slug_taken(&final_slug) {
            final_slug = format!("{}-{}", base_slug, counter);
            counter += 1;
        }
    }

    let db = state.db.lock().unwrap();
    
    // Check if user already exists
    if let Ok(Some(_)) = db.get_user_by_email(&req.email) {
        return Json(serde_json::json!({"ok": false, "error": "Email is already registered"}));
    }

    let current_max = db.get_max_port().unwrap_or(Some(state.base_port)).unwrap_or(state.base_port);
    let new_port = std::cmp::max(current_max, state.base_port) + 1;

    // Create User first (status=pending — needs Super Admin approval)
    let user_id = match db.create_user(&req.email, &hash, "admin", None) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({"ok": false, "error": sanitize_error(&format!("Failed to create user: {e}"))})),
    };
    
    // Set user status to pending
    let _ = db.update_user_status(&user_id, "pending");

    // Create tenant with owner_id linking to the user (tenant stays stopped until approved)
    match db.create_tenant(&req.company_name, &final_slug, new_port, "openai", "gpt-4o-mini", "free", Some(&user_id)) {
        Ok(tenant) => {
            // Update user's tenant_id
            let _ = db.update_user_tenant(&user_id, Some(&tenant.id));
            db.log_event("saas_registration", "user", &user_id, Some(&format!("tenant={},status=pending", tenant.slug))).ok();
            
            Json(serde_json::json!({
                "ok": true, 
                "slug": final_slug, 
                "message": "Đăng ký thành công! Tài khoản đang chờ duyệt bởi Admin. Bạn sẽ nhận được thông báo khi được kích hoạt."
            }))
        }
        Err(e) => {
            // Rollback: delete the user if tenant creation fails
            let _ = db.delete_user_cascade(&user_id);
            Json(serde_json::json!({"ok": false, "error": sanitize_error(&format!("Failed to create tenant: {e}"))}))
        }
    }
}

pub async fn change_password_handler(
    State(state): State<Arc<AdminState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<ChangePasswordReq>,
) -> Json<serde_json::Value> {
    // Password strength validation (unified)
    if req.new_password.len() < MIN_PASSWORD_LENGTH {
        return Json(serde_json::json!({"ok": false, "error": format!("Mật khẩu mới phải có ít nhất {} ký tự", MIN_PASSWORD_LENGTH)}));
    }

    let current_user_opt = {
        let db = state.db.lock().unwrap();
        db.get_user_by_email(&claims.email)
    };
    
    if let Ok(Some((id, old_hash, _))) = current_user_opt {
        let current_password = req.current_password.clone();
        let is_valid = tokio::task::spawn_blocking(move || crate::auth::verify_password(&current_password, &old_hash)).await.unwrap_or(false);
        
        if is_valid {
            let new_pwd = req.new_password.clone();
            if let Ok(Ok(new_hash)) = tokio::task::spawn_blocking(move || crate::auth::hash_password(&new_pwd)).await {
                let db = state.db.lock().unwrap();
                if db.update_user_password(&id, &new_hash).is_ok() {
                    db.log_event("password_changed", "user", &id, None).ok();
                    return Json(serde_json::json!({"ok": true}));
                }
            }
            return Json(serde_json::json!({"ok": false, "error": "Could not update password"}));
        }
        return Json(serde_json::json!({"ok": false, "error": "Incorrect current password"}));
    }
    Json(serde_json::json!({"ok": false, "error": "User not found"}))
}

pub async fn forgot_password_handler(
    State(state): State<Arc<AdminState>>,
    Json(req): Json<ForgotPasswordReq>,
) -> Json<serde_json::Value> {
    // C3 FIX: Rate limiting — max 3 password reset requests per email per 15 minutes
    {
        let mut attempts = state.register_attempts.lock().unwrap(); // Reuse register_attempts for reset
        let key = format!("reset:{}", req.email);
        let now = std::time::Instant::now();
        if let Some((count, first_at)) = attempts.get(&key) {
            if now.duration_since(*first_at).as_secs() < 900 && *count >= 3 {
                // Note: Still return OK to prevent email enumeration
                tracing::warn!("[security] Password reset rate limit hit for {}", req.email);
                return Json(serde_json::json!({"ok": true, "message": "If this email is registered, a reset link will be sent."}));
            }
            if now.duration_since(*first_at).as_secs() >= 900 {
                attempts.remove(&key);
            }
        }
        let entry = attempts.entry(key).or_insert((0, now));
        entry.0 += 1;
    }

    // Generate secure token
    let token = format!("{}-{}", uuid::Uuid::new_v4().to_string(), uuid::Uuid::new_v4().to_string());
    let expires_at = chrono::Utc::now().timestamp() + 3600; // 1 hour validity

    {
        let db = state.db.lock().unwrap();
        if let Ok(Some(_)) = db.get_user_by_email(&req.email) {
            db.save_password_reset_token(&req.email, &token, expires_at).ok();
            
            let smtp_host = db.get_platform_config("smtp.host").unwrap_or_default();
            let smtp_user = db.get_platform_config("smtp.user").unwrap_or_default();
            let smtp_pass = db.get_platform_config("smtp.pass").unwrap_or_default();
            
            // SMTP implementation via lettre
            if !smtp_host.is_empty() && !smtp_user.is_empty() {
                tokio::spawn(async move {
                    use lettre::{Message, SmtpTransport, Transport};
                    use lettre::transport::smtp::authentication::Credentials;
                    
                    let from_addr = match smtp_user.parse() {
                        Ok(a) => a,
                        Err(e) => { tracing::warn!("SMTP from address invalid: {e}"); return; }
                    };
                    let to_addr = match req.email.parse() {
                        Ok(a) => a,
                        Err(e) => { tracing::warn!("SMTP to address invalid: {e}"); return; }
                    };
                    let email = match Message::builder()
                        .from(from_addr)
                        .to(to_addr)
                        .subject("BizClaw Password Reset")
                        .body(format!("Reset your password here: https://apps.bizclaw.vn/#/reset-password?token={}", token))
                    {
                        Ok(e) => e,
                        Err(e) => { tracing::warn!("Failed to build email: {e}"); return; }
                    };

                    let creds = Credentials::new(smtp_user, smtp_pass);
                    // L6 FIX: Handle SMTP relay error instead of unwrap()
                    let mailer = match SmtpTransport::relay(&smtp_host) {
                        Ok(m) => m.credentials(creds).build(),
                        Err(e) => {
                            tracing::error!("[security] SMTP relay error for host '{}': {e}", smtp_host);
                            return;
                        }
                    };
                        
                    match mailer.send(&email) {
                        Ok(_) => tracing::info!("Password reset email sent successfully"),
                        Err(e) => tracing::warn!("[security] Failed to send password reset email: {e}"),
                    }
                });
            } else {
                tracing::warn!("SMTP is not configured — password reset token generated but cannot be sent. Configure SMTP in platform settings.");
            }
            
            // Note: Even if user is not found, we return OK to prevent email enumeration
        }
    }
    Json(serde_json::json!({"ok": true, "message": "If this email is registered, a reset link will be sent."}))
}

pub async fn reset_password_handler(
    State(state): State<Arc<AdminState>>,
    Json(req): Json<ResetPasswordReq>,
) -> Json<serde_json::Value> {
    // Password strength validation (unified)
    if req.new_password.len() < MIN_PASSWORD_LENGTH {
        return Json(serde_json::json!({"ok": false, "error": format!("Mật khẩu phải có ít nhất {} ký tự", MIN_PASSWORD_LENGTH)}));
    }

    let reset_info = {
        let db = state.db.lock().unwrap();
        match db.get_password_reset_email(&req.token) {
            Ok(email) => {
                if let Ok(Some((id, _, _))) = db.get_user_by_email(&email) {
                    Some((email, id))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    };

    if let Some((email, id)) = reset_info {
        let new_pwd = req.new_password.clone();
        if let Ok(Ok(hash)) = tokio::task::spawn_blocking(move || crate::auth::hash_password(&new_pwd)).await {
            let db = state.db.lock().unwrap();
            if db.update_user_password(&id, &hash).is_ok() {
                db.delete_password_reset_token(&email).ok();
                db.log_event("password_reset", "user", &id, None).ok();
                return Json(serde_json::json!({"ok": true}));
            }
        }
        return Json(serde_json::json!({"ok": false, "error": "Failed to update password"}));
    }
    Json(serde_json::json!({"ok": false, "error": "Invalid or expired token"}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.co.vn"));
        assert!(is_valid_email("a@b.co"));
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("@.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("user@com"));
        assert!(!is_valid_email("user @example.com"));
        assert!(!is_valid_email("@@@..."));
        assert!(!is_valid_email("a@b.c")); // TLD too short
    }

    #[test]
    fn test_slug_generation() {
        assert_eq!(generate_safe_slug("My Company"), "my-company");
        assert_eq!(generate_safe_slug("Hello World 123"), "hello-world-123");
        assert!(!generate_safe_slug("admin").starts_with("admin"));
        assert!(!generate_safe_slug("").is_empty());
    }
}
