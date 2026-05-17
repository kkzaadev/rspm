//! Log file helpers for RSPM.

pub mod rotator;
pub mod tail;
pub mod timestamp;
pub mod writer;

pub use tail::tail_file;
pub use writer::{LogOpts, LogWriter};
