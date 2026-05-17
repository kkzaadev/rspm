//! SSH deploy placeholder.

use rspm_core::{Result, RspmError};

/// Connects to a remote host.
pub fn connect(host: &str) -> Result<()> {
    Err(RspmError::Unsupported(format!(
        "ssh deploy to {host} is not implemented yet"
    )))
}
