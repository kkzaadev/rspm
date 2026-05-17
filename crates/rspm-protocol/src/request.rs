//! Request messages sent from clients to the daemon.

use serde::{Deserialize, Serialize};

use rspm_core::types::AppConfig;

/// A target process selector.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Selector {
    /// Selects every process.
    All,
    /// Selects a process by PM2-compatible ID.
    Id(u32),
    /// Selects processes by app name.
    Name(String),
}

impl Selector {
    /// Parses CLI input into a selector.
    ///
    /// ```
    /// let selector = rspm_protocol::Selector::parse("all");
    /// assert_eq!(selector, rspm_protocol::Selector::All);
    /// ```
    pub fn parse(value: &str) -> Self {
        if value == "all" {
            return Self::All;
        }

        match value.parse::<u32>() {
            Ok(id) => Self::Id(id),
            Err(_) => Self::Name(value.to_owned()),
        }
    }
}

/// Daemon request message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Request {
    /// Starts one normalized app config.
    Start { app: Box<AppConfig> },
    /// Stops matching processes.
    Stop { selector: Selector },
    /// Restarts matching processes.
    Restart { selector: Selector },
    /// Soft reload (zero-downtime rolling restart for cluster mode).
    Reload { selector: Selector },
    /// Deletes matching processes.
    Delete { selector: Selector },
    /// Lists process state.
    List,
    /// Reads log tail lines.
    Logs {
        /// Optional process selector.
        selector: Option<Selector>,
        /// Number of lines to return.
        lines: usize,
    },
    /// Saves current process list to disk.
    Save,
    /// Reloads the dump file and starts every persisted app.
    Resurrect,
    /// Returns daemon liveness.
    Ping,
    /// Returns daemon version.
    GetVersion,
    /// Sends a signal to matching processes.
    SendSignal {
        /// Process selector.
        selector: Selector,
        /// Signal name such as `SIGTERM`.
        signal: String,
    },
    /// Shuts down the daemon.
    KillDaemon,
}
