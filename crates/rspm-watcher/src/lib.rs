//! File watcher boundary for future PM2 watch parity.

pub mod debounce;
pub mod matcher;
pub mod watcher;

pub use watcher::{AppWatcher, WatchEvent};
