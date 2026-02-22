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
    #[cfg(windows)]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.arg("/C").arg(command);
        c
    };
    
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("sh");
        c.arg("-c").arg(command);
        c
    };
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_info() {
        let info = RuntimeInfo::current();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.pid > 0);
    }

    #[tokio::test]
    async fn test_execute_echo() {
        let (stdout, stderr, code) = execute_with_stderr("echo hello", None).await.unwrap();
        assert_eq!(stdout.trim(), "hello");
        assert!(stderr.is_empty());
        assert_eq!(code, 0);
    }
}
