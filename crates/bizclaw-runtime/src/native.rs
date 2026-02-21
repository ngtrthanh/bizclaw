//! Native runtime module — process management utilities.

use std::process::Stdio;
use bizclaw_core::error::Result;

/// Runtime environment information.
pub struct RuntimeInfo {
    pub os: String,
    pub arch: String,
    pub pid: u32,
}

impl RuntimeInfo {
    /// Gather current runtime information.
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            pid: std::process::id(),
        }
    }
}

/// Execute a command and capture both stdout and stderr.
pub async fn execute_with_stderr(
    command: &str,
    workdir: Option<&str>,
) -> Result<(String, String, i32)> {
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}
