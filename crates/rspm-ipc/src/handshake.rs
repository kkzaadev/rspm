//! Protocol handshake helpers.

use rspm_core::{Result, RspmError};
use rspm_protocol::version::PROTOCOL_VERSION;

/// Validates a peer protocol version.
///
/// ```
/// assert!(rspm_ipc::handshake::validate_protocol_version(1).is_ok());
/// ```
pub fn validate_protocol_version(peer_version: u32) -> Result<()> {
    if peer_version == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(RspmError::Protocol(format!(
            "protocol version mismatch: local={}, peer={}",
            PROTOCOL_VERSION, peer_version
        )))
    }
}
