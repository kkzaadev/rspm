//! Configuration loading and normalization for RSPM.

pub mod env_expand;
pub mod format;
pub mod loader;
pub mod normalize;
pub mod schema;

pub use loader::load_file;
pub use normalize::apply_defaults;
pub use schema::{AppConfigInput, ConfigDocument};
