//! Unix domain socket IPC for RSPM.

pub mod bus;
pub mod client;
pub mod codec;
pub mod handshake;
pub mod server;

pub use bus::PubSubBus;
pub use client::{EventSubscriber, IpcClient};
pub use server::{BoxedRequestFuture, IpcServer, RequestHandler};
