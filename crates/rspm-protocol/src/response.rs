//! Response messages sent from the daemon to clients.

use serde::{Deserialize, Serialize};

use rspm_core::types::ProcessInfo;

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
