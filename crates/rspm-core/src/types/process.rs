//! Process state types.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// PM2-compatible process ID type.
pub type ProcessId = u32;

/// PM2-compatible process status values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    /// Process is online.
    #[serde(rename = "online")]
    Online,
    /// Process is currently stopping.
    #[serde(rename = "stopping")]
    Stopping,
    /// Process is stopped.
    #[serde(rename = "stopped")]
    Stopped,
    /// Process failed.
    #[serde(rename = "errored")]
    Errored,
    /// PM2 one-launch status.
    #[serde(rename = "one-launch-status")]
    OneLaunchStatus,
    /// Process is launching.
    #[serde(rename = "launching")]
    Launching,
    /// Process is waiting for a restart.
    #[serde(rename = "waiting restart")]
    Waiting,
}

impl ProcessStatus {
    /// Returns true when this status means a live process is expected.
    ///
    /// ```
    /// assert!(rspm_core::types::ProcessStatus::Online.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Online | Self::Launching)
    }
}

/// Public process information returned to clients.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Monotonic process ID.
    pub pm_id: ProcessId,
    /// Application name.
    pub name: String,
    /// Operating system PID.
    pub pid: Option<u32>,
    /// Process status.
    pub status: ProcessStatus,
    /// Script path.
    pub script: PathBuf,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Process creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Number of restarts performed.
    pub restart_time: u32,
    /// Number of unstable restarts.
    pub unstable_restarts: u32,
    /// Timestamp when the process became online.
    pub pm_uptime: Option<DateTime<Utc>>,
    /// Stdout log file.
    pub out_file: Option<PathBuf>,
    /// Stderr log file.
    pub error_file: Option<PathBuf>,
    /// Latest CPU usage percent (smoothed via [`crate::types::CpuSample`]).
    #[serde(default)]
    pub cpu_percent: f32,
    /// Latest resident set size in bytes.
    #[serde(default)]
    pub memory_bytes: u64,
}

impl ProcessInfo {
    /// Creates a stopped process info value for an app.
    ///
    /// ```
    /// let app = rspm_core::types::AppConfig::from_script("server.js", None);
    /// let info = rspm_core::types::ProcessInfo::new(0, &app);
    /// assert_eq!(info.pm_id, 0);
    /// ```
    pub fn new(pm_id: ProcessId, app: &crate::types::AppConfig) -> Self {
        Self {
            pm_id,
            name: app.name.clone(),
            pid: None,
            status: ProcessStatus::Stopped,
            script: app.script.clone(),
            cwd: app.cwd.clone(),
            created_at: Utc::now(),
            restart_time: 0,
            unstable_restarts: 0,
            pm_uptime: None,
            out_file: app.out_file.clone(),
            error_file: app.error_file.clone(),
            cpu_percent: 0.0,
            memory_bytes: 0,
        }
    }
}
