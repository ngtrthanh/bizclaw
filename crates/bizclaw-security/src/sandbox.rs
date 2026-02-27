//! Process sandboxing for secure tool execution.
//!
//! Provides isolation mechanisms for tool execution to prevent
//! accidental or malicious system modifications.

use bizclaw_core::error::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Sandbox configuration for tool execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Working directory for sandboxed processes
    pub workdir: PathBuf,
    /// Maximum execution time
    pub timeout: Duration,
    /// Maximum output size in bytes
    pub max_output_bytes: usize,
    /// Environment variables to pass through
    pub env_passthrough: Vec<String>,
    /// Whether to use filesystem isolation (chroot-like)
    pub isolate_fs: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            timeout: Duration::from_secs(30),
            max_output_bytes: 1024 * 1024, // 1MB
            env_passthrough: vec!["PATH".into(), "HOME".into(), "USER".into()],
            isolate_fs: false,
        }
    }
}

/// Sandbox for executing tools with resource limits.
pub struct Sandbox {
    config: SandboxConfig,
}

impl Sandbox {
    /// Create a new sandbox with default config.
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    /// Create a sandbox with custom config.
    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Execute a command within the sandbox.
    pub async fn execute(&self, command: &str) -> Result<SandboxResult> {
        let command = command.to_string();
        let workdir = self.config.workdir.clone();
        let env_passthrough = self.config.env_passthrough.clone();
        let timeout = self.config.timeout;
        let max_output_bytes = self.config.max_output_bytes;

        let result = tokio::task::spawn_blocking(move || {
            let mut cmd = std::process::Command::new("sh");
            cmd.arg("-c").arg(&command);
            cmd.current_dir(&workdir);

            // Clear environment and only pass through allowed vars
            cmd.env_clear();
            for var in &env_passthrough {
                if let Ok(val) = std::env::var(var) {
                    cmd.env(var, val);
                }
            }

            cmd.output()
        });

        // Execute with timeout
        let output = tokio::time::timeout(timeout, result)
            .await
            .map_err(|_| {
                bizclaw_core::error::BizClawError::Timeout(format!(
                    "Command timed out after {:?}",
                    timeout
                ))
            })?
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(e.to_string()))?
            .map_err(|e| bizclaw_core::error::BizClawError::Tool(e.to_string()))?;

        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Truncate if exceeding max output
        if stdout.len() > max_output_bytes {
            stdout.truncate(max_output_bytes);
            stdout.push_str("\n... [output truncated]");
        }
        if stderr.len() > max_output_bytes {
            stderr.truncate(max_output_bytes);
            stderr.push_str("\n... [output truncated]");
        }

        Ok(SandboxResult {
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        })
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a sandboxed execution.
#[derive(Debug)]
pub struct SandboxResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}
