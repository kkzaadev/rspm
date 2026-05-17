//! Binds TCP listening sockets with `SO_REUSEPORT` for universal cluster mode.
//!
//! Unlike PM2 which leans on Node.js's `cluster` module, we want a runtime
//! that lets any language scale by re-binding the same port from N children.
//! Linux's `SO_REUSEPORT` gives us in-kernel load balancing across all
//! processes that bind the same `(addr, port)` pair — so as long as each
//! child sets the option, the kernel hands out incoming connections evenly.

use std::net::SocketAddr;
use std::os::fd::{IntoRawFd, RawFd};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use rspm_core::Result;

/// Binds a TCP listening socket with `SO_REUSEADDR` + `SO_REUSEPORT` and
/// returns its raw file descriptor.
///
/// The returned descriptor is owned by the caller, who is responsible for
/// closing it (or wrapping it in `TcpListener::from_raw_fd`).
///
/// ```no_run
/// use std::net::SocketAddr;
/// # fn demo() -> rspm_core::Result<()> {
/// let fd = rspm_cluster::bind_reuseport("127.0.0.1:0".parse::<SocketAddr>().unwrap())?;
/// assert!(fd >= 0);
/// # Ok(())
/// # }
/// ```
pub fn bind_reuseport(addr: SocketAddr) -> Result<RawFd> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&SockAddr::from(addr))?;
    socket.listen(1024)?;
    Ok(socket.into_raw_fd())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::{FromRawFd, OwnedFd};

    #[test]
    fn binds_loopback_twice() {
        // Pick an ephemeral port via first bind.
        let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("probe bind");
        let addr = probe.local_addr().expect("local addr");
        drop(probe);

        let fd1 = bind_reuseport(addr).expect("first bind");
        let fd2 = bind_reuseport(addr).expect("second bind");
        assert!(fd1 >= 0 && fd2 >= 0);

        // Wrap in OwnedFd so the kernel descriptors close on drop.
        let _ = unsafe { OwnedFd::from_raw_fd(fd1) };
        let _ = unsafe { OwnedFd::from_raw_fd(fd2) };
    }
}
