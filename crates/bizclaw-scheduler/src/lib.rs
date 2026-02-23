//! # BizClaw Scheduler
//!
//! Ultra-lightweight task scheduler and notification system.
//! Inspired by PicoClaw's file-based state and ZeroClaw's <10ms cold start.
//!
//! ## Design Principles (for 512MB RAM devices)
//! - No external dependencies (no Redis, no RabbitMQ)
//! - File-based persistence (JSON) — markdown-readable state
//! - Tokio timers only — zero overhead when idle
//! - Notification routing — pick the best channel to reach user
//!
//! ## Architecture
//! ```text
//! Scheduler (tokio interval)
//!   ├── CronTask: "0 8 * * *" → "Tóm tắt email"
//!   ├── OnceTask: "2026-02-22 15:00" → "Họp team"
//!   ├── IntervalTask: every 30min → "Check server"
//!   └── on trigger → NotificationRouter
//!                      ├── Telegram (priority 1)
//!                      ├── Discord (priority 2)
//!                      └── Email (priority 3)
//! ```

pub mod cron;
pub mod engine;
pub mod notify;
pub mod store;
pub mod tasks;

pub use engine::SchedulerEngine;
pub use notify::{Notification, NotifyChannel, NotifyRouter};
pub use store::TaskStore;
pub use tasks::{Task, TaskStatus, TaskType};
