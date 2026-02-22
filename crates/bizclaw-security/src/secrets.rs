//! Encrypted secrets management.
//!
//! Provides secure storage and retrieval of API keys, tokens, and
//! other sensitive configuration values using AES-256-GCM encryption
//! (authenticated encryption with nonce) with a machine-specific key
//! derived from hostname + username.
//!
//! Migration: Automatically detects and upgrades legacy AES-256-ECB files.

use aes::Aes256;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::Aead;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bizclaw_core::error::{BizClawError, Result};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// GCM nonce size (12 bytes, standard for AES-GCM).
const GCM_NONCE_SIZE: usize = 12;

/// Magic prefix for GCM-encrypted data (to distinguish from legacy ECB).
const GCM_MAGIC: &[u8; 4] = b"BCGM";

/// Manages encrypted secrets stored on disk.
pub struct SecretStore {
    secrets: HashMap<String, String>,
    secrets_path: PathBuf,
    encrypt: bool,
    key: [u8; 32],
}

impl SecretStore {
    /// Create a new secret store.
    pub fn new(encrypt: bool) -> Self {
        let secrets_path = bizclaw_core::config::BizClawConfig::home_dir().join("secrets.enc");
        Self {
            secrets: HashMap::new(),
            secrets_path,
            encrypt,
            key: derive_machine_key(),
        }
    }

    /// Load secrets from disk.
    pub fn load(&mut self) -> Result<()> {
        if !self.secrets_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.secrets_path)?;

        let json_str = if self.encrypt {
            let raw = BASE64.decode(content.trim())
                .map_err(|e| BizClawError::Security(format!("Base64 decode failed: {e}")))?;

            // Try GCM first (new format), fallback to ECB (legacy migration)
            match decrypt_aes256_gcm(&raw, &self.key) {
                Ok(decrypted) => {
                    String::from_utf8(decrypted)
                        .map_err(|e| BizClawError::Security(format!("Decryption produced invalid UTF-8: {e}")))?
                }
                Err(_) => {
                    // Legacy ECB migration path
                    tracing::warn!("Migrating secrets from legacy AES-256-ECB to AES-256-GCM");
                    let decrypted = decrypt_aes256_ecb(&raw, &self.key);
                    let json = String::from_utf8(decrypted)
                        .map_err(|e| BizClawError::Security(format!("Legacy decryption produced invalid UTF-8: {e}")))?;

                    // Parse to validate, then re-save will use GCM
                    self.secrets = serde_json::from_str(&json)
                        .map_err(|e| BizClawError::Security(format!("Failed to parse secrets: {e}")))?;

                    // Re-encrypt with GCM immediately
                    tracing::info!("Re-encrypting secrets with AES-256-GCM");
                    self.save()?;

                    tracing::info!("Loaded {} secrets (migrated to GCM)", self.secrets.len());
                    return Ok(());
                }
            }
        } else {
            content
        };

        self.secrets = serde_json::from_str(&json_str)
            .map_err(|e| BizClawError::Security(format!("Failed to parse secrets: {e}")))?;

        tracing::info!("Loaded {} secrets from {}", self.secrets.len(), self.secrets_path.display());
        Ok(())
    }

    /// Save secrets to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.secrets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.secrets)?;

        let content = if self.encrypt {
            let encrypted = encrypt_aes256_gcm(json.as_bytes(), &self.key)?;
            BASE64.encode(&encrypted)
        } else {
            json
        };

        // Set restrictive permissions on Unix (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true).create(true).truncate(true).mode(0o600)
                .open(&self.secrets_path)?;
            file.write_all(content.as_bytes())?;
            return Ok(());
        }

        #[cfg(not(unix))]
        {
            std::fs::write(&self.secrets_path, content)?;
            Ok(())
        }
    }

    /// Get a secret value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.secrets.get(key).map(|s| s.as_str())
    }

    /// Set a secret value.
    pub fn set(&mut self, key: &str, value: &str) {
        self.secrets.insert(key.to_string(), value.to_string());
    }

    /// Remove a secret.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.secrets.remove(key)
    }

    /// List all secret keys (without values).
    pub fn keys(&self) -> Vec<&str> {
        self.secrets.keys().map(|k| k.as_str()).collect()
    }

    /// Load from a specific path.
    pub fn load_from(path: &Path) -> Result<Self> {
        let mut store = Self {
            secrets: HashMap::new(),
            secrets_path: path.to_path_buf(),
            encrypt: false,
            key: derive_machine_key(),
        };
        store.load()?;
        Ok(store)
    }
}

/// Derive a machine-specific AES-256 key from hostname + username.
fn derive_machine_key() -> [u8; 32] {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "bizclaw".into());
    let username = whoami::username();
    let salt = format!("bizclaw::{username}@{hostname}::secrets");

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

// ── AES-256-GCM (primary, authenticated) ──────────────────────

/// AES-256-GCM encrypt: BCGM magic + 12-byte nonce + ciphertext.
fn encrypt_aes256_gcm(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; GCM_NONCE_SIZE];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| BizClawError::Security(format!("AES-256-GCM encryption failed: {e}")))?;

    // Format: BCGM (4) + nonce (12) + ciphertext
    let mut result = Vec::with_capacity(4 + GCM_NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(GCM_MAGIC);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// AES-256-GCM decrypt: expects BCGM magic + 12-byte nonce + ciphertext.
fn decrypt_aes256_gcm(data: &[u8], key: &[u8; 32]) -> std::result::Result<Vec<u8>, String> {
    if data.len() < 4 + GCM_NONCE_SIZE {
        return Err("Data too short for GCM".into());
    }

    // Check magic
    if &data[0..4] != GCM_MAGIC {
        return Err("Not a GCM-encrypted payload (missing BCGM magic)".into());
    }

    let nonce = Nonce::from_slice(&data[4..4 + GCM_NONCE_SIZE]);
    let ciphertext = &data[4 + GCM_NONCE_SIZE..];

    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("AES-256-GCM decryption failed: {e}"))
}

// ── AES-256-ECB (legacy, kept for migration only) ─────────────

/// Legacy AES-256-ECB decrypt with PKCS7 unpadding.
/// Only used for migrating old secrets.enc files.
fn decrypt_aes256_ecb(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(key));
    let block_size = 16;

    let mut decrypted = Vec::with_capacity(data.len());
    for chunk in data.chunks(block_size) {
        if chunk.len() == block_size {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            decrypted.extend_from_slice(&block);
        }
    }

    // Remove PKCS7 padding
    if let Some(&pad_len) = decrypted.last() {
        let pad_len = pad_len as usize;
        if pad_len <= block_size && pad_len <= decrypted.len() {
            let valid = decrypted[decrypted.len() - pad_len..]
                .iter()
                .all(|&b| b == pad_len as u8);
            if valid {
                decrypted.truncate(decrypted.len() - pad_len);
            }
        }
    }

    decrypted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcm_encrypt_decrypt_roundtrip() {
        let key = derive_machine_key();
        let data = b"Hello, BizClaw secrets!";
        let encrypted = encrypt_aes256_gcm(data, &key).unwrap();
        let decrypted = decrypt_aes256_gcm(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_gcm_different_nonces() {
        let key = derive_machine_key();
        let data = b"same plaintext";
        let enc1 = encrypt_aes256_gcm(data, &key).unwrap();
        let enc2 = encrypt_aes256_gcm(data, &key).unwrap();
        // Two encryptions of the same data should produce different ciphertexts
        assert_ne!(enc1, enc2, "GCM should produce different ciphertexts due to random nonces");
        // But both should decrypt to the same plaintext
        assert_eq!(decrypt_aes256_gcm(&enc1, &key).unwrap(), data);
        assert_eq!(decrypt_aes256_gcm(&enc2, &key).unwrap(), data);
    }

    #[test]
    fn test_gcm_tamper_detection() {
        let key = derive_machine_key();
        let data = b"sensitive data";
        let mut encrypted = encrypt_aes256_gcm(data, &key).unwrap();
        // Tamper with the last byte of ciphertext
        if let Some(last) = encrypted.last_mut() {
            *last ^= 0xFF;
        }
        // Should fail to decrypt (authentication failure)
        assert!(decrypt_aes256_gcm(&encrypted, &key).is_err());
    }

    #[test]
    fn test_legacy_ecb_detection() {
        // Data without BCGM magic should fail GCM and trigger ECB path
        let fake_ecb_data = vec![0u8; 32]; // not starting with BCGM
        assert!(decrypt_aes256_gcm(&fake_ecb_data, &[0u8; 32]).is_err());
    }

    #[test]
    fn test_secret_store_operations() {
        let mut store = SecretStore::new(false);
        store.set("api_key", "sk-test-12345");
        store.set("bot_token", "123456:ABC-DEF");

        assert_eq!(store.get("api_key"), Some("sk-test-12345"));
        assert_eq!(store.get("bot_token"), Some("123456:ABC-DEF"));
        assert_eq!(store.get("missing"), None);

        assert!(store.keys().contains(&"api_key"));
        assert_eq!(store.remove("api_key"), Some("sk-test-12345".into()));
        assert_eq!(store.get("api_key"), None);
    }
}
