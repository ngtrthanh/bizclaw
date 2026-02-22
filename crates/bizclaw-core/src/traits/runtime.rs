//! Runtime adapter trait.

use crate::error::Result;
use async_trait::async_trait;

/// Runtime adapter for command execution environments.
#[async_trait]
pub trait RuntimeAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn execute_command(&self, command: &str, workdir: Option<&str>) -> Result<String>;
}
