//! Default values used by config normalization and daemon behavior.

use std::time::Duration;

/// PM2 default worker interval in milliseconds.
pub const WORKER_INTERVAL_MS: u64 = 30_000;
/// PM2 default graceful kill timeout in milliseconds.
pub const KILL_TIMEOUT_MS: u64 = 1_600;
/// PM2 default listen timeout in milliseconds.
pub const LISTEN_TIMEOUT_MS: u64 = 8_000;
/// PM2 default max restart count.
pub const MAX_RESTARTS: u32 = 16;
/// PM2 default instance environment variable name.
pub const DEFAULT_INSTANCE_VAR: &str = "NODE_APP_INSTANCE";
/// Default process minimum uptime in milliseconds.
pub const MIN_UPTIME_MS: u64 = 1_000;
/// Exponential backoff cap for restart delay (PM2 parity).
pub const EXP_BACKOFF_CAP_MS: u64 = 15_000;
/// Multiplier applied to `prev_restart_delay` on each restart.
pub const EXP_BACKOFF_MULTIPLIER_NUM: u64 = 3;
/// Denominator paired with [`EXP_BACKOFF_MULTIPLIER_NUM`] (3/2 = 1.5x).
pub const EXP_BACKOFF_MULTIPLIER_DEN: u64 = 2;
/// Uptime after which `prev_restart_delay` resets back to base.
pub const EXP_BACKOFF_RESET_TIMER_MS: u64 = 30_000;
/// Default fixed restart delay in milliseconds (no delay).
pub const RESTART_DELAY_MS: u64 = 0;
/// Default maximum bytes per log file before rotation kicks in.
pub const LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// Default number of rotated archives kept per stream.
pub const LOG_MAX_ARCHIVES: usize = 10;
/// Default pub/sub bus capacity for daemon events (log lines, process state).
pub const PUB_BUS_CAPACITY: usize = 1024;

/// Returns the default auto-restart flag.
///
/// ```
/// assert!(rspm_core::defaults::auto_restart());
/// ```
pub fn auto_restart() -> bool {
    true
}

/// Returns the default watch flag.
///
/// ```
/// assert!(!rspm_core::defaults::watch());
/// ```
pub fn watch() -> bool {
    false
}

/// Returns the default kill timeout in milliseconds.
///
/// ```
/// assert_eq!(rspm_core::defaults::kill_timeout_ms(), 1600);
/// ```
pub fn kill_timeout_ms() -> u64 {
    KILL_TIMEOUT_MS
}

/// Returns the default listen timeout in milliseconds.
///
/// ```
/// assert_eq!(rspm_core::defaults::listen_timeout_ms(), 8000);
/// ```
pub fn listen_timeout_ms() -> u64 {
    LISTEN_TIMEOUT_MS
}

/// Returns the default min uptime in milliseconds.
///
/// ```
/// assert_eq!(rspm_core::defaults::min_uptime_ms(), 1000);
/// ```
pub fn min_uptime_ms() -> u64 {
    MIN_UPTIME_MS
}

/// Returns the default max restart count.
///
/// ```
/// assert_eq!(rspm_core::defaults::max_restarts(), 16);
/// ```
pub fn max_restarts() -> u32 {
    MAX_RESTARTS
}

/// Returns the default instance environment variable name.
///
/// ```
/// assert_eq!(rspm_core::defaults::instance_var(), "NODE_APP_INSTANCE");
/// ```
pub fn instance_var() -> String {
    DEFAULT_INSTANCE_VAR.to_owned()
}

/// Returns the default worker interval.
///
/// ```
/// assert_eq!(rspm_core::defaults::worker_interval().as_millis(), 30000);
/// ```
pub fn worker_interval() -> Duration {
    Duration::from_millis(WORKER_INTERVAL_MS)
}

/// Returns the default fixed restart delay in milliseconds.
///
/// ```
/// assert_eq!(rspm_core::defaults::restart_delay_ms(), 0);
/// ```
pub fn restart_delay_ms() -> u64 {
    RESTART_DELAY_MS
}

/// Returns the exponential backoff cap (in ms).
///
/// ```
/// assert_eq!(rspm_core::defaults::exp_backoff_cap_ms(), 15_000);
/// ```
pub fn exp_backoff_cap_ms() -> u64 {
    EXP_BACKOFF_CAP_MS
}

/// Returns the uptime threshold that resets `prev_restart_delay`.
///
/// ```
/// assert_eq!(rspm_core::defaults::exp_backoff_reset_timer_ms(), 30_000);
/// ```
pub fn exp_backoff_reset_timer_ms() -> u64 {
    EXP_BACKOFF_RESET_TIMER_MS
}

/// Computes the next exponential backoff delay using the PM2 1.5x rule
/// capped at [`EXP_BACKOFF_CAP_MS`]. `base` is the initial delay configured
/// on the app (`exp_backoff_restart_delay`).
///
/// ```
/// use rspm_core::defaults::next_exp_backoff;
/// assert_eq!(next_exp_backoff(0, 100), 100);
/// assert_eq!(next_exp_backoff(100, 100), 150);
/// assert_eq!(next_exp_backoff(20_000, 100), 15_000);
/// ```
pub fn next_exp_backoff(prev: u64, base: u64) -> u64 {
    if prev == 0 {
        return base.min(EXP_BACKOFF_CAP_MS);
    }
    let scaled = prev.saturating_mul(EXP_BACKOFF_MULTIPLIER_NUM) / EXP_BACKOFF_MULTIPLIER_DEN;
    scaled.min(EXP_BACKOFF_CAP_MS)
}

/// Returns the default maximum bytes per log file.
///
/// ```
/// assert_eq!(rspm_core::defaults::log_max_bytes(), 10 * 1024 * 1024);
/// ```
pub fn log_max_bytes() -> u64 {
    LOG_MAX_BYTES
}

/// Returns the default number of log archives retained.
///
/// ```
/// assert_eq!(rspm_core::defaults::log_max_archives(), 10);
/// ```
pub fn log_max_archives() -> usize {
    LOG_MAX_ARCHIVES
}

/// Returns the default pub/sub bus capacity.
///
/// ```
/// assert_eq!(rspm_core::defaults::pub_bus_capacity(), 1024);
/// ```
pub fn pub_bus_capacity() -> usize {
    PUB_BUS_CAPACITY
}
