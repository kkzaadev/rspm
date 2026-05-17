//! Application configuration types.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::constants::{CLUSTER_MODE_ID, FORK_MODE_ID};
use crate::defaults;
use crate::types::EnvMap;

/// Execution mode for an app.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// PM2-style fork mode.
    #[default]
    #[serde(alias = "fork")]
    ForkMode,
    /// PM2-style cluster mode.
    #[serde(alias = "cluster")]
    ClusterMode,
}

impl ExecutionMode {
    /// Returns the PM2 wire/config identifier for this mode.
    ///
    /// ```
    /// assert_eq!(rspm_core::types::ExecutionMode::ForkMode.as_pm2_str(), "fork_mode");
    /// ```
    pub fn as_pm2_str(&self) -> &'static str {
        match self {
            Self::ForkMode => FORK_MODE_ID,
            Self::ClusterMode => CLUSTER_MODE_ID,
        }
    }
}

/// Instance count requested by an app config.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstanceCount {
    /// Exact instance count.
    Count(u32),
    /// String form such as `max` or `-1`.
    Named(String),
}

impl Default for InstanceCount {
    fn default() -> Self {
        Self::Count(1)
    }
}

impl InstanceCount {
    /// Resolves the configured value to an instance count.
    ///
    /// ```
    /// let instances = rspm_core::types::InstanceCount::Named("max".to_owned());
    /// assert!(instances.resolve() >= 1);
    /// ```
    pub fn resolve(&self) -> u32 {
        let cpus = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        let cpus = u32::try_from(cpus).unwrap_or(u32::MAX).max(1);

        match self {
            Self::Count(0) => 1,
            Self::Count(value) => *value,
            Self::Named(value) if value == "max" => cpus,
            Self::Named(value) if value == "-1" => cpus.saturating_sub(1).max(1),
            Self::Named(value) => value
                .parse::<u32>()
                .ok()
                .filter(|count| *count > 0)
                .unwrap_or(1),
        }
    }
}

/// File watch configuration accepted by PM2.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WatchSpec {
    /// Enables or disables watching the app cwd.
    Enabled(bool),
    /// Explicit paths to watch.
    Paths(Vec<String>),
}

impl Default for WatchSpec {
    fn default() -> Self {
        Self::Enabled(defaults::watch())
    }
}

/// Normalized application configuration used by the daemon.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application display name.
    pub name: String,
    /// Script or executable path.
    pub script: PathBuf,
    /// Arguments passed after the script.
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the process.
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Execution mode.
    #[serde(default, alias = "exec_mode")]
    pub execution_mode: ExecutionMode,
    /// Number of instances.
    #[serde(default)]
    pub instances: InstanceCount,
    /// Memory restart threshold, preserved as PM2-style text for now.
    #[serde(default)]
    pub max_memory_restart: Option<String>,
    /// Whether crashed apps should be restarted.
    #[serde(default = "defaults::auto_restart", alias = "autorestart")]
    pub auto_restart: bool,
    /// Watch configuration.
    #[serde(default)]
    pub watch: WatchSpec,
    /// Paths ignored by watcher.
    #[serde(default)]
    pub ignore_watch: Vec<String>,
    /// Graceful kill timeout in milliseconds.
    #[serde(default = "defaults::kill_timeout_ms")]
    pub kill_timeout_ms: u64,
    /// Minimum uptime in milliseconds.
    #[serde(default = "defaults::min_uptime_ms")]
    pub min_uptime_ms: u64,
    /// Maximum restart count.
    #[serde(default = "defaults::max_restarts")]
    pub max_restarts: u32,
    /// Environment variables added to the child.
    #[serde(default)]
    pub env: EnvMap,
    /// Named environment overrides from `env_*` blocks.
    #[serde(default)]
    pub env_overrides: BTreeMap<String, EnvMap>,
    /// Stderr log file.
    #[serde(default, alias = "error_file")]
    pub error_file: Option<PathBuf>,
    /// Stdout log file.
    #[serde(default, alias = "out_file")]
    pub out_file: Option<PathBuf>,
    /// Combined stdout/stderr log file.
    #[serde(default, alias = "log_file")]
    pub combined_file: Option<PathBuf>,
    /// Optional log date format.
    #[serde(default)]
    pub log_date_format: Option<String>,
    /// Whether logs are merged across instances.
    #[serde(default)]
    pub merge_logs: bool,
    /// Whether RSPM should prefix raw logs with timestamps.
    #[serde(default, alias = "time")]
    pub prefix_timestamp: bool,
    /// Cron expression for restart.
    #[serde(default)]
    pub cron_restart: Option<String>,
    /// Optional interpreter path.
    #[serde(default, alias = "exec_interpreter")]
    pub interpreter: Option<PathBuf>,
    /// Arguments passed to the interpreter.
    #[serde(default, alias = "node_args", alias = "interpreterArgs")]
    pub interpreter_args: Vec<String>,
    /// Environment variable name used for the instance index.
    #[serde(default = "defaults::instance_var")]
    pub instance_var: String,
    /// Whether the daemon waits for a ready event.
    #[serde(default)]
    pub wait_ready: bool,
    /// Listen timeout in milliseconds.
    #[serde(default = "defaults::listen_timeout_ms")]
    pub listen_timeout_ms: u64,
    /// Fixed delay (ms) between exit and the next restart attempt.
    ///
    /// Equivalent to PM2 `restart_delay`. Zero disables the fixed delay.
    #[serde(default = "defaults::restart_delay_ms")]
    pub restart_delay_ms: u64,
    /// Initial delay (ms) used to seed the exponential backoff sequence.
    ///
    /// Equivalent to PM2 `exp_backoff_restart_delay`. When `None`, the
    /// daemon falls back to [`AppConfig::restart_delay_ms`] without any
    /// backoff escalation. Subsequent restarts multiply the previous delay
    /// by 1.5x capped at [`crate::defaults::EXP_BACKOFF_CAP_MS`].
    #[serde(default)]
    pub exp_backoff_restart_delay_ms: Option<u64>,
    /// Exit codes that are considered intentional. When a child exits with
    /// one of these codes the daemon marks it `Stopped` and skips restart
    /// even when `auto_restart` is true. Mirrors PM2 `stop_exit_codes`.
    #[serde(default)]
    pub stop_exit_codes: Vec<i32>,
}

impl AppConfig {
    /// Returns the configured `max_memory_restart` value resolved into bytes.
    ///
    /// Returns `None` when the field is empty or malformed.
    ///
    /// ```
    /// use rspm_core::types::AppConfig;
    /// let mut app = AppConfig::from_script("server.js", None);
    /// app.max_memory_restart = Some("200M".to_owned());
    /// assert_eq!(app.max_memory_bytes(), Some(200 * 1024 * 1024));
    /// ```
    pub fn max_memory_bytes(&self) -> Option<u64> {
        self.max_memory_restart
            .as_ref()
            .and_then(|raw| crate::types::byte_size::parse_byte_size(raw))
    }

    /// Creates a minimal app config from a script path.
    ///
    /// ```
    /// let app = rspm_core::types::AppConfig::from_script("server.js", None);
    /// assert_eq!(app.name, "server");
    /// ```
    pub fn from_script(path: impl Into<PathBuf>, name: Option<String>) -> Self {
        let script = path.into();
        let fallback = infer_name(&script);
        Self {
            name: name.unwrap_or(fallback),
            script,
            args: Vec::new(),
            cwd: None,
            execution_mode: ExecutionMode::ForkMode,
            instances: InstanceCount::Count(1),
            max_memory_restart: None,
            auto_restart: defaults::auto_restart(),
            watch: WatchSpec::default(),
            ignore_watch: Vec::new(),
            kill_timeout_ms: defaults::kill_timeout_ms(),
            min_uptime_ms: defaults::min_uptime_ms(),
            max_restarts: defaults::max_restarts(),
            env: EnvMap::new(),
            env_overrides: BTreeMap::new(),
            error_file: None,
            out_file: None,
            combined_file: None,
            log_date_format: None,
            merge_logs: false,
            prefix_timestamp: false,
            cron_restart: None,
            interpreter: None,
            interpreter_args: Vec::new(),
            instance_var: defaults::instance_var(),
            wait_ready: false,
            listen_timeout_ms: defaults::listen_timeout_ms(),
            restart_delay_ms: defaults::restart_delay_ms(),
            exp_backoff_restart_delay_ms: None,
            stop_exit_codes: Vec::new(),
        }
    }
}

/// Infers a PM2-like app name from a script path.
///
/// ```
/// assert_eq!(rspm_core::types::app::infer_name(std::path::Path::new("api.js")), "api");
/// ```
pub fn infer_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("app")
        .to_owned()
}
