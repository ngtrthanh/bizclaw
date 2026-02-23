//! # BizClaw Memory
//! Memory and persistence backends with 3-tier brain architecture

pub mod brain;
pub mod noop;
pub mod sqlite;
pub mod vector;

use bizclaw_core::config::MemoryConfig;
use bizclaw_core::error::Result;
use bizclaw_core::traits::MemoryBackend;

/// Create a memory backend from configuration.
pub fn create_memory(config: &MemoryConfig) -> Result<Box<dyn MemoryBackend>> {
    match config.backend.as_str() {
        "sqlite" => Ok(Box::new(sqlite::SqliteMemory::new()?)),
        "none" => Ok(Box::new(noop::NoopMemory)),
        other => Err(bizclaw_core::error::BizClawError::Memory(format!(
            "Unknown memory backend: {other}"
        ))),
    }
}
