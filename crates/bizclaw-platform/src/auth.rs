//! JWT authentication for admin panel.

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// JWT claims.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user ID
    pub email: String,
    pub role: String,
    pub exp: usize,
}

/// Generate a JWT token.
pub fn create_token(
    user_id: &str,
    email: &str,
    role: &str,
    secret: &str,
) -> Result<String, String> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.into(),
        email: email.into(),
        role: role.into(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Token creation failed: {e}"))
}

/// Validate and decode a JWT token.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, String> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Token validation failed: {e}"))
}

/// Hash a password using bcrypt.
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, 12).map_err(|e| format!("Hash error: {e}"))
}

/// Verify a password against a bcrypt hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_roundtrip() {
        let secret = "test-secret-key-bizclaw";
        let token = create_token("user-1", "admin@test.com", "admin", secret).unwrap();
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.email, "admin@test.com");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_invalid_token() {
        let result = validate_token("invalid.token.here", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash() {
        let hash = hash_password("MySecurePassword123!").unwrap();
        assert!(verify_password("MySecurePassword123!", &hash));
        assert!(!verify_password("WrongPassword", &hash));
    }
}
