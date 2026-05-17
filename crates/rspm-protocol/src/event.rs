//! Pub/sub event messages reserved for daemon notifications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use rspm_core::types::{ProcessId, ProcessInfo};

/// Log stream identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStream {
    /// Stdout stream.
    Out,
    /// Stderr stream.
    Err,
}

/// Event emitted by the daemon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum Event {
    /// Process became online.
    ProcessOnline { process: ProcessInfo },
    /// Process exited.
    ProcessExit {
        /// Process ID.
        pm_id: ProcessId,
        /// Exit code if available.
        code: Option<i32>,
    },
    /// Process message event.
    ProcessMsg {
        /// Process ID.
        pm_id: ProcessId,
        /// Raw JSON payload.
        payload: serde_json::Value,
    },
    /// Log line event.
    Log {
        /// Process ID.
        pm_id: ProcessId,
        /// App name.
        name: String,
        /// Stream.
        stream: LogStream,
        /// Log data.
        data: String,
        /// Event timestamp.
        at: DateTime<Utc>,
    },
    /// System warning.
    SystemWarn { message: String },
}
