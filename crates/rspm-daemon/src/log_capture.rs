//! Log capture boundary.

/// Returns true because child stdout/stderr are redirected by the supervisor.
pub fn child_stdio_is_captured() -> bool {
    true
}
