//! Web Dashboard — embedded HTML/JS served at /
//!
//! Self-contained SPA for managing BizClaw.

/// Dashboard HTML page.
pub fn dashboard_html() -> &'static str {
    include_str!("dashboard.html")
}
