//! Debounce window helpers for file watch events.

use std::time::Duration;

/// Default debounce window. Matches PM2's typical chokidar settling time.
///
/// ```
/// assert_eq!(rspm_watcher::debounce::default_delay().as_millis(), 200);
/// ```
pub fn default_delay() -> Duration {
    Duration::from_millis(200)
}
