//! Error types shared across RSPM crates.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias used by RSPM library crates.
pub type Result<T> = std::result::Result<T, RspmError>;

/// Shared error enum for recoverable RSPM failures.
#[derive(Debug, Error)]
pub enum RspmError {
    /// Filesystem or operating system I/O failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or parsing failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing failed.
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML parsing failed.
    #[error("toml decode error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization failed.
    #[error("toml encode error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// A protocol frame or request was invalid.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// A config file or app definition was invalid.
    #[error("config error: {0}")]
    Config(String),

    /// A requested entity was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A requested feature exists in the roadmap but is not implemented yet.
    #[error("unsupported feature: {0}")]
    Unsupported(String),

    /// Daemon lifecycle or supervision failed.
    #[error("daemon error: {0}")]
    Daemon(String),

    /// A process signal failed.
    #[error("signal error: {0}")]
    Signal(String),

    /// A path could not be represented in the expected format.
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
}
