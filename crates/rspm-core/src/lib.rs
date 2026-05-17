//! Shared foundation types for RSPM.

pub mod constants;
pub mod defaults;
pub mod error;
pub mod paths;
pub mod types;
pub mod version;

pub use error::{Result, RspmError};
