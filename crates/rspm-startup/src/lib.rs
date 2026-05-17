//! Startup script generation.

pub mod detect;
pub mod generator;

pub use detect::{InitSystem, detect_init_system};
pub use generator::{StartupCtx, generate};
