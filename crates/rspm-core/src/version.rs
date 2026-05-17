//! Version helpers.

/// Returns the crate package version.
///
/// ```
/// assert!(!rspm_core::version::pkg_version().is_empty());
/// ```
pub fn pkg_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns a compact build info string.
///
/// ```
/// assert!(rspm_core::version::build_info().contains("rspm"));
/// ```
pub fn build_info() -> String {
    format!("rspm {}", pkg_version())
}
