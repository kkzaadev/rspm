//! Module lifecycle placeholder.

use rspm_core::{Result, RspmError};

/// Starts a module.
pub fn start(name: &str) -> Result<()> {
    Err(RspmError::Unsupported(format!(
        "module lifecycle for {name} is not implemented yet"
    )))
}
