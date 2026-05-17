//! HTTP server placeholder.

use rspm_core::{Result, RspmError};

/// Starts the HTTP API.
pub async fn serve(port: u16) -> Result<()> {
    Err(RspmError::Unsupported(format!(
        "http api on port {port} is not implemented yet"
    )))
}
