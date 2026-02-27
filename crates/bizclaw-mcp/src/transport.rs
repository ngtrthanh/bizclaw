//! Stdio transport for MCP — spawns a child process and communicates via JSON-RPC.
//!
//! NOTE: This module is currently disabled to avoid Docker signal handling issues.
//! MCP functionality will be re-enabled in a future release with a signal-free implementation.

use std::collections::HashMap;

use crate::types::{JsonRpcRequest, JsonRpcResponse};

/// Stdio transport — manages a child process for JSON-RPC communication.
pub struct StdioTransport {
    _phantom: std::marker::PhantomData<()>,
}

impl StdioTransport {
    /// Spawn a new MCP server process.
    pub async fn spawn(
        _command: &str,
        _args: &[String],
        _env: &HashMap<String, String>,
    ) -> Result<Self, String> {
        Err("MCP is temporarily disabled due to Docker signal handling issues. Will be re-enabled in a future release.".to_string())
    }

    /// Send a JSON-RPC request and read the response.
    pub(crate) async fn request(
        &mut self,
        _req: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse, String> {
        Err("MCP is temporarily disabled".to_string())
    }

    /// Check if the child process is still running.
    pub fn is_alive(&mut self) -> bool {
        false
    }

    /// Kill the child process.
    pub async fn shutdown(&mut self) {
        // No-op
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // No-op
    }
}
