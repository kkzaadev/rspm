//! Module installer placeholder.

use rspm_core::{Result, RspmError};

/// Installs a module source.
pub fn install(source: &str) -> Result<()> {
    Err(RspmError::Unsupported(format!(
        "module install for {source} is not implemented yet"
    )))
}
