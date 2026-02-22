//! Command and path allowlist management.
//!
//! Controls which shell commands can be executed and which filesystem paths
//! can be accessed by the agent.

use bizclaw_core::config::AutonomyConfig;
use std::collections::HashSet;

/// Manages command and path allowlists for security enforcement.
pub struct Allowlist {
    allowed_commands: HashSet<String>,
    forbidden_paths: Vec<String>,
    workspace_only: bool,
}

impl Allowlist {
    /// Create a new allowlist from autonomy configuration.
    pub fn new(config: &AutonomyConfig) -> Self {
        Self {
            allowed_commands: config.allowed_commands.iter().cloned().collect(),
            forbidden_paths: config.forbidden_paths.clone(),
            workspace_only: config.workspace_only,
        }
    }

    /// Check if a command is allowed to execute.
    pub fn is_command_allowed(&self, command: &str) -> bool {
        let cmd_base = command.split_whitespace().next().unwrap_or("");
        // Also check the basename (in case full path is given)
        let cmd_name = std::path::Path::new(cmd_base)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(cmd_base);

        self.allowed_commands.contains(cmd_name) || self.allowed_commands.contains(cmd_base)
    }

    /// Check if a path is allowed to access.
    pub fn is_path_allowed(&self, path: &str) -> bool {
        let expanded = shellexpand::tilde(path).to_string();
        let canonical = std::path::Path::new(&expanded);

        // Check against forbidden paths
        for forbidden in &self.forbidden_paths {
            let exp_forbidden = shellexpand::tilde(forbidden).to_string();
            if expanded.starts_with(&exp_forbidden) {
                return false;
            }
        }

        // If workspace_only, restrict to workspace directory
        if self.workspace_only
            && let Ok(cwd) = std::env::current_dir() {
                return canonical.starts_with(&cwd)
                    || expanded.starts_with(&cwd.to_string_lossy().to_string());
            }

        true
    }

    /// Add a command to the allowlist.
    pub fn allow_command(&mut self, command: &str) {
        self.allowed_commands.insert(command.to_string());
    }

    /// Remove a command from the allowlist.
    pub fn deny_command(&mut self, command: &str) {
        self.allowed_commands.remove(command);
    }

    /// Add a path to the forbidden list.
    pub fn forbid_path(&mut self, path: &str) {
        self.forbidden_paths.push(path.to_string());
    }
}
