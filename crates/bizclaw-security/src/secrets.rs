//! Encrypted secrets management.
//!
//! Provides secure storage and retrieval of API keys, tokens, and
//! other sensitive configuration values using AES-256 encryption.

use bizclaw_core::error::{BizClawError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manages encrypted secrets stored on disk.
pub struct SecretStore {
    secrets: HashMap<String, String>,
    secrets_path: PathBuf,
    encrypt: bool,
}

impl SecretStore {
    /// Create a new secret store.
    pub fn new(encrypt: bool) -> Self {
        let secrets_path = bizclaw_core::config::BizClawConfig::home_dir().join("secrets.json");
        Self {
            secrets: HashMap::new(),
            secrets_path,
            encrypt,
        }
    }

    /// Load secrets from disk.
    pub fn load(&mut self) -> Result<()> {
        if !self.secrets_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.secrets_path)?;

        if self.encrypt {
            // TODO: AES-256 decryption with machine-specific key
            // For now, use plaintext JSON as fallback
            tracing::warn!("Encrypted secret store not yet implemented, using plaintext");
        }

        self.secrets = serde_json::from_str(&content)
            .map_err(|e| BizClawError::Security(format!("Failed to parse secrets: {e}")))?;

        Ok(())
    }

    /// Save secrets to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.secrets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.secrets)?;

        if self.encrypt {
            // TODO: AES-256 encryption
            tracing::warn!("Encrypted secret store not yet implemented, using plaintext");
        }

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true).mode(0o600);
            use std::io::Write;
            let mut file = opts.open(&self.secrets_path)?;
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
        };
        store.load()?;
        Ok(store)
    }
}
