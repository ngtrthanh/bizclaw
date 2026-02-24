//! # BizClaw Platform
//!
//! Multi-tenant management platform — run multiple BizClaw agents on a single VPS.
//! Includes admin dashboard, tenant lifecycle management, pairing security,
//! subdomain routing, resource monitoring, and audit logging.

pub mod admin;
pub mod auth;
pub mod config;
pub mod db;
pub mod tenant;
pub mod self_serve;

pub use admin::AdminServer;
pub use db::PlatformDb;
pub use tenant::TenantManager;
