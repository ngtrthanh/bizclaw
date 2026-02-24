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
    if req.email.is_empty() || req.password.is_empty() || req.company_name.is_empty() {
        return Json(serde_json::json!({"ok": false, "error": "Email, password, and company name are required"}));
    }

    let password = req.password.clone();
    let hash = match tokio::task::spawn_blocking(move || crate::auth::hash_password(&password)).await.unwrap_or_else(|e| Err(e.to_string())) {
        Ok(h) => h,
        Err(e) => return Json(serde_json::json!({"ok": false, "error": format!("Hash error: {e}")})),
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
        Err(e) => return Json(serde_json::json!({"ok": false, "error": format!("Failed to create user: {e}")})),
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
            Json(serde_json::json!({"ok": false, "error": format!("Failed to create tenant: {e}")}))
        }
    }
}

pub async fn change_password_handler(
    State(state): State<Arc<AdminState>>,
    Extension(claims): Extension<crate::auth::Claims>,
    Json(req): Json<ChangePasswordReq>,
) -> Json<serde_json::Value> {
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
            
            // SMTP implementation via lettre goes here (async or blocking).
            if !smtp_host.is_empty() && !smtp_user.is_empty() {
                // To avoid blocking, we can send it in a thread
                tokio::spawn(async move {
                    use lettre::{Message, SmtpTransport, Transport};
                    use lettre::transport::smtp::authentication::Credentials;
                    
                    let email = Message::builder()
                        .from(smtp_user.parse().unwrap())
                        .to(req.email.parse().unwrap())
                        .subject("BizClaw Password Reset")
                        .body(format!("Reset your password here: https://apps.bizclaw.vn/#/reset-password?token={}", token))
                        .unwrap();

                    let creds = Credentials::new(smtp_user, smtp_pass);
                    let mailer = SmtpTransport::relay(&smtp_host)
                        .unwrap()
                        .credentials(creds)
                        .build();
                        
                    let _ = mailer.send(&email);
                });
            } else {
                tracing::warn!("SMTP is not configured! Token generated: {}", token);
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
