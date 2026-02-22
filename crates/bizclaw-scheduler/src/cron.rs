//! Lightweight cron expression parser.
//! Supports: "MIN HOUR DOM MON DOW" (5-field, no seconds)
//! Wildcards: *, */N, N
//! Example: "0 8 * * *" = every day at 8:00
//!
//! Designed for PicoClaw-level simplicity — no cron crate dependency.

use chrono::{DateTime, Duration, Timelike, Utc};

/// Parse a simple cron expression and compute the next run time.
pub fn next_run_from_cron(expression: &str, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() != 5 {
        tracing::warn!(
            "Invalid cron expression: '{}' (need 5 fields: MIN HOUR DOM MON DOW)",
            expression
        );
        return None;
    }

    let minute_spec = parts[0];
    let hour_spec = parts[1];
    let _dom_spec = parts[2]; // Day of month (simplified: only * supported)
    let _mon_spec = parts[3]; // Month (simplified: only * supported)
    let _dow_spec = parts[4]; // Day of week (simplified: only * supported)

    // Parse minute
    let minutes = parse_field(minute_spec, 0, 59)?;
    let hours = parse_field(hour_spec, 0, 23)?;

    // Find next matching time after `after`
    let mut candidate = after + Duration::minutes(1);
    // Zero out seconds
    candidate = candidate.with_second(0).unwrap_or(candidate);

    // Try up to 48 hours ahead
    for _ in 0..(48 * 60) {
        let m = candidate.minute();
        let h = candidate.hour();

        if minutes.contains(&m) && hours.contains(&h) {
            return Some(candidate);
        }
        candidate += Duration::minutes(1);
    }

    None
}

/// Parse a cron field into a list of matching values.
fn parse_field(field: &str, min: u32, max: u32) -> Option<Vec<u32>> {
    if field == "*" {
        return Some((min..=max).collect());
    }

    // */N — every N
    if let Some(step) = field.strip_prefix("*/") {
        let n: u32 = step.parse().ok()?;
        if n == 0 {
            return None;
        }
        return Some((min..=max).step_by(n as usize).collect());
    }

    // Comma-separated: "0,15,30,45"
    if field.contains(',') {
        let vals: Result<Vec<u32>, _> = field.split(',').map(|s| s.trim().parse()).collect();
        return vals
            .ok()
            .map(|v| v.into_iter().filter(|x| *x >= min && *x <= max).collect());
    }

    // Single number
    let n: u32 = field.parse().ok()?;
    if n >= min && n <= max {
        Some(vec![n])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_every_hour() {
        let after = Utc.with_ymd_and_hms(2026, 2, 22, 10, 30, 0).unwrap();
        let next = next_run_from_cron("0 * * * *", after).unwrap();
        assert_eq!(next.hour(), 11);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn test_specific_time() {
        let after = Utc.with_ymd_and_hms(2026, 2, 22, 7, 0, 0).unwrap();
        let next = next_run_from_cron("0 8 * * *", after).unwrap();
        assert_eq!(next.hour(), 8);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn test_every_15_minutes() {
        let after = Utc.with_ymd_and_hms(2026, 2, 22, 10, 2, 0).unwrap();
        let next = next_run_from_cron("*/15 * * * *", after).unwrap();
        assert_eq!(next.minute(), 15);
    }

    #[test]
    fn test_invalid_expression() {
        let after = Utc::now();
        assert!(next_run_from_cron("bad", after).is_none());
    }
}
