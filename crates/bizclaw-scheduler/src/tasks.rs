//! Task definitions — the core data model for scheduled work.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A scheduled task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// What to do when triggered (prompt to send to Agent, or action).
    pub action: TaskAction,
    /// When/how to trigger.
    pub task_type: TaskType,
    /// Current status.
    pub status: TaskStatus,
    /// Notification channel preference (where to send result).
    pub notify_via: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last triggered timestamp.
    pub last_run: Option<DateTime<Utc>>,
    /// Next scheduled run.
    pub next_run: Option<DateTime<Utc>>,
    /// How many times this task has run.
    pub run_count: u32,
    /// Whether the task is enabled.
    pub enabled: bool,
}

/// What the task does when triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    /// Send a prompt to the Agent and get a response.
    AgentPrompt(String),
    /// Send a fixed notification message.
    Notify(String),
    /// Execute a webhook URL.
    Webhook {
        url: String,
        method: String,
        body: Option<String>,
        #[serde(default)]
        headers: Vec<(String, String)>,
    },
}

/// How/when the task triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// Run once at a specific time.
    Once { at: DateTime<Utc> },
    /// Run on a cron schedule (lightweight cron expression).
    Cron { expression: String },
    /// Run every N seconds.
    Interval { every_secs: u64 },
}

/// Task status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Disabled,
}

impl Task {
    /// Create a new one-time task.
    pub fn once(name: &str, at: DateTime<Utc>, action: TaskAction) -> Self {
        Self {
            id: uuid_v4(),
            name: name.to_string(),
            action,
            task_type: TaskType::Once { at },
            status: TaskStatus::Pending,
            notify_via: None,
            created_at: Utc::now(),
            last_run: None,
            next_run: Some(at),
            run_count: 0,
            enabled: true,
        }
    }

    /// Create a recurring interval task.
    pub fn interval(name: &str, every_secs: u64, action: TaskAction) -> Self {
        let next = Utc::now() + chrono::Duration::seconds(every_secs as i64);
        Self {
            id: uuid_v4(),
            name: name.to_string(),
            action,
            task_type: TaskType::Interval { every_secs },
            status: TaskStatus::Pending,
            notify_via: None,
            created_at: Utc::now(),
            last_run: None,
            next_run: Some(next),
            run_count: 0,
            enabled: true,
        }
    }

    /// Create a cron-scheduled task.
    pub fn cron(name: &str, expression: &str, action: TaskAction) -> Self {
        Self {
            id: uuid_v4(),
            name: name.to_string(),
            action,
            task_type: TaskType::Cron {
                expression: expression.to_string(),
            },
            status: TaskStatus::Pending,
            notify_via: None,
            created_at: Utc::now(),
            last_run: None,
            next_run: None, // Computed by cron parser
            run_count: 0,
            enabled: true,
        }
    }

    /// Check if this task should run now.
    pub fn should_run(&self) -> bool {
        if !self.enabled || self.status == TaskStatus::Disabled {
            return false;
        }
        match &self.next_run {
            Some(next) => Utc::now() >= *next,
            None => false,
        }
    }
}

/// Simple UUID v4 generator (no external crate needed for Pi).
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("task-{:x}-{:x}", t.as_secs(), t.subsec_nanos())
}
