//! Timestamp formatting helpers for log prefixes.
//!
//! PM2 prefixes each line with `log_date_format` (a strftime-style pattern)
//! when an app sets `time: true`. We model the same with chrono's `format`
//! helpers and default to RFC3339-ish output when the user only opts in.

use chrono::Utc;

/// Default format used when an app enables `prefix_timestamp` without
/// specifying `log_date_format`. PM2's default is RFC3339-ish.
pub const DEFAULT_LOG_DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

/// Formats the current UTC timestamp as RFC 3339 for compatibility.
///
/// ```
/// assert!(!rspm_logs::timestamp::now_iso8601().is_empty());
/// ```
pub fn now_iso8601() -> String {
    Utc::now().to_rfc3339()
}

/// Formats the current UTC timestamp using a chrono strftime pattern.
///
/// Returns the formatted timestamp. An empty pattern degrades to the empty
/// string so callers can detect "no prefix".
///
/// ```
/// assert_eq!(rspm_logs::timestamp::format_now("%Y").len(), 4);
/// ```
pub fn format_now(pattern: &str) -> String {
    if pattern.is_empty() {
        return String::new();
    }
    Utc::now().format(pattern).to_string()
}

/// Prefixes one log line with an ISO-8601 timestamp followed by a single space.
///
/// ```
/// let line = rspm_logs::timestamp::prefix_line("hello");
/// assert!(line.ends_with(" hello"));
/// ```
pub fn prefix_line(line: &str) -> String {
    format!("{} {}", now_iso8601(), line)
}

/// Prefixes a line using a strftime pattern; returns the line unchanged when
/// the pattern is empty.
///
/// ```
/// let line = rspm_logs::timestamp::prefix_with("%Y", "hello");
/// assert!(line.ends_with(" hello"));
/// ```
pub fn prefix_with(pattern: &str, line: &str) -> String {
    if pattern.is_empty() {
        return line.to_owned();
    }
    format!("{} {}", format_now(pattern), line)
}
