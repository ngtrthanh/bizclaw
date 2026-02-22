//! Encrypted secrets management.
//!
//! Provides secure storage and retrieval of API keys, tokens, and
//! other sensitive configuration values using AES-256-ECB encryption
//! with a machine-specific key derived from hostname + username.

use aes::Aes256;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bizclaw_core::error::{BizClawError, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
            // Decrypt from base64 → AES-256 → JSON
            let encrypted = BASE64
                .decode(content.trim())
                .map_err(|e| BizClawError::Security(format!("Base64 decode failed: {e}")))?;
            let decrypted = decrypt_aes256(&encrypted, &self.key);
            String::from_utf8(decrypted).map_err(|e| {
                BizClawError::Security(format!("Decryption produced invalid UTF-8: {e}"))
            })?
        } else {
            content
        };

        self.secrets = serde_json::from_str(&json_str)
            .map_err(|e| BizClawError::Security(format!("Failed to parse secrets: {e}")))?;

        tracing::debug!(
            "Loaded {} secrets from {}",
            self.secrets.len(),
            self.secrets_path.display()
        );
        Ok(())
    }

    /// Save secrets to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.secrets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.secrets)?;

        let content = if self.encrypt {
            // Encrypt: JSON → AES-256 → base64
            let encrypted = encrypt_aes256(json.as_bytes(), &self.key);
            BASE64.encode(&encrypted)
        } else {
            json
        };

        // Set restrictive permissions on Unix (0600)
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&self.secrets_path)?;
            file.write_all(content.as_bytes())?;
            Ok(())
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

/// AES-256-ECB encrypt with PKCS7 padding.
fn encrypt_aes256(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256::new(GenericArray::from_slice(key));
    let block_size = 16;

    // PKCS7 padding
    let padding_len = block_size - (data.len() % block_size);
    let mut padded = data.to_vec();
    padded.extend(std::iter::repeat_n(padding_len as u8, padding_len));

    let mut encrypted = Vec::with_capacity(padded.len());
    for chunk in padded.chunks(block_size) {
        let mut block = GenericArray::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }

    encrypted
}

/// AES-256-ECB decrypt with PKCS7 unpadding.
fn decrypt_aes256(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
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
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_machine_key();
        let data = b"Hello, BizClaw secrets!";
        let encrypted = encrypt_aes256(data, &key);
        let decrypted = decrypt_aes256(&encrypted, &key);
        assert_eq!(decrypted, data);
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
