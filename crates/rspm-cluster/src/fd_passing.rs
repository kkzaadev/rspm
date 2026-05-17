//! File descriptor passing placeholder.

use std::os::fd::RawFd;
use std::os::unix::net::UnixStream;

use rspm_core::{Result, RspmError};

/// Sends a file descriptor to a child over a Unix stream.
pub fn send_fd_to_child(_socket: &UnixStream, fd: RawFd) -> Result<()> {
    Err(RspmError::Unsupported(format!(
        "SCM_RIGHTS fd passing for fd {fd} is not implemented yet"
    )))
}
