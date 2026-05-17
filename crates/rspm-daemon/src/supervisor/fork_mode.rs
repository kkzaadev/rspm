//! Fork mode notes.

/// Returns the PM2-compatible fork mode label.
pub fn label() -> &'static str {
    rspm_core::constants::FORK_MODE_ID
}
