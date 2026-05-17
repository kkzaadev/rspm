//! Response messages sent from the daemon to clients.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use rspm_core::types::{ProcessId, ProcessInfo};

/// One row in a [`Response::Describe`] payload. Mirrors the subset of fields
/// PM2 prints from `pm2 describe <id>`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessDetail {
    /// The summary fields shown by `rspm list`.
    pub info: ProcessInfo,
    /// Script arguments after the entrypoint.
    pub args: Vec<String>,
    /// Interpreter path, if explicitly set.
    pub interpreter: Option<PathBuf>,
    /// Execution mode (`fork_mode` or `cluster_mode`).
    pub exec_mode: String,
    /// Resolved instance count for the parent app.
    pub instances: u32,
    /// Effective merged env (after `env_*` overrides).
    pub env: BTreeMap<String, String>,
    /// Auto-restart enabled.
    pub auto_restart: bool,
    /// Max-restarts ceiling.
    pub max_restarts: u32,
    /// Min uptime in ms before a restart is considered stable.
    pub min_uptime_ms: u64,
    /// Configured kill timeout in ms.
    pub kill_timeout_ms: u64,
    /// Configured fixed restart delay in ms.
    pub restart_delay_ms: u64,
    /// Exponential-backoff base (None = disabled).
    pub exp_backoff_restart_delay_ms: Option<u64>,
    /// `max_memory_restart` raw string ("200M", "1G", ...) if configured.
    pub max_memory_restart: Option<String>,
    /// Stop exit codes that are treated as intentional.
    pub stop_exit_codes: Vec<i32>,
    /// Watch enabled (true if watch is on or a paths list is non-empty).
    pub watch: bool,
}

/// Daemon response message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum Response {
    /// Generic success response.
    Ack { message: String },
    /// Started processes.
    Started { processes: Vec<ProcessInfo> },
    /// Process list response.
    ProcessList { processes: Vec<ProcessInfo> },
    /// Single process response.
    Process { process: ProcessInfo },
    /// Detailed process descriptions (one per matched id).
    Describe { details: Vec<ProcessDetail> },
    /// Env maps for matched processes, keyed by pm_id.
    Env {
        /// Per-process effective env map.
        envs: BTreeMap<ProcessId, BTreeMap<String, String>>,
    },
    /// Log lines response.
    Logs { lines: Vec<String> },
    /// Ping response.
    Pong { msg: String },
    /// Version response.
    Version { version: String },
    /// Error response.
    Error { message: String },
}

impl Response {
    /// Creates an error response from any displayable message.
    ///
    /// ```
    /// let response = rspm_protocol::Response::error("bad request");
    /// assert!(matches!(response, rspm_protocol::Response::Error { .. }));
    /// ```
    pub fn error(message: impl ToString) -> Self {
        Self::Error {
            message: message.to_string(),
        }
    }

    /// Returns this response as a result, mapping protocol errors to `RspmError`.
    ///
    /// ```
    /// let response = rspm_protocol::Response::Ack { message: "ok".into() };
    /// assert!(response.into_result().is_ok());
    /// ```
    pub fn into_result(self) -> rspm_core::Result<Self> {
        match self {
            Self::Error { message } => Err(rspm_core::RspmError::Daemon(message)),
            response => Ok(response),
        }
    }
}
