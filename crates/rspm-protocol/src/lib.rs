//! Wire protocol definitions for RSPM IPC.

pub mod event;
pub mod frame;
pub mod request;
pub mod response;
pub mod version;

pub use event::{Event, LogStream};
pub use request::{Request, Selector};
pub use response::{ProcessDetail, Response};
pub use version::PROTOCOL_VERSION;
