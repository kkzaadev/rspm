//! Cluster mode notes.

/// Returns the PM2-compatible cluster mode label.
pub fn label() -> &'static str {
    rspm_core::constants::CLUSTER_MODE_ID
}
