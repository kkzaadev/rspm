//! Lifecycle helpers.

use std::time::Duration;

/// Computes a restart delay for the current v0.0.1 policy.
pub fn restart_delay() -> Duration {
    Duration::from_millis(0)
}
