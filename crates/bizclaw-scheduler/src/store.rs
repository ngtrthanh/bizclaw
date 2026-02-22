//! File-based task store — PicoClaw-inspired persistence.
//! Tasks saved as JSON files — human-readable, git-friendly.
//! Zero overhead: only reads/writes on task changes, not on every tick.

use crate::tasks::Task;
use std::path::{Path, PathBuf};

/// File-based task store.
pub struct TaskStore {
    path: PathBuf,
}

impl TaskStore {
    /// Create a new task store at the given directory.
    pub fn new(dir: &Path) -> Self {
        std::fs::create_dir_all(dir).ok();
        Self {
            path: dir.to_path_buf(),
        }
    }

    /// Default store path (~/.bizclaw/scheduler/tasks.json).
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".bizclaw").join("scheduler")
    }

    /// Save all tasks to disk.
    pub fn save(&self, tasks: &[Task]) -> Result<(), String> {
        let file = self.path.join("tasks.json");
        let json =
            serde_json::to_string_pretty(tasks).map_err(|e| format!("Serialize error: {e}"))?;
        std::fs::write(&file, &json).map_err(|e| format!("Write error: {e}"))?;
        tracing::debug!("💾 Saved {} tasks to {}", tasks.len(), file.display());
        Ok(())
    }

    /// Load tasks from disk.
    pub fn load(&self) -> Vec<Task> {
        let file = self.path.join("tasks.json");
        if !file.exists() {
            return Vec::new();
        }
        match std::fs::read_to_string(&file) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                tracing::warn!("⚠️ Failed to parse tasks.json: {e}");
                Vec::new()
            }),
            Err(e) => {
                tracing::warn!("⚠️ Failed to read tasks.json: {e}");
                Vec::new()
            }
        }
    }
}
