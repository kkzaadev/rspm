//! Serde schema for pre-normalized app config files.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use rspm_core::types::{ExecutionMode, InstanceCount, WatchSpec};

/// String-or-array config field used by PM2 for args and interpreter args.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringList {
    /// A shell-like whitespace separated string.
    String(String),
    /// A list of strings.
    List(Vec<String>),
}

impl StringList {
    /// Converts this value into a vector of strings.
    ///
    /// ```
    /// let args = rspm_config::schema::StringList::String("--port 3000".into());
    /// assert_eq!(args.into_vec(), vec!["--port", "3000"]);
    /// ```
    pub fn into_vec(self) -> Vec<String> {
        match self {
            Self::String(value) => value
                .split_whitespace()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
            Self::List(values) => values,
        }
    }
}

/// Raw app config before defaults and validation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppConfigInput {
    /// Application name.
    #[serde(default)]
    pub name: Option<String>,
    /// Script path.
    #[serde(default, alias = "exec")]
    pub script: Option<PathBuf>,
    /// Process args.
    #[serde(default)]
    pub args: Option<StringList>,
    /// Working directory.
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    /// Execution mode.
    #[serde(default, alias = "exec_mode")]
    pub execution_mode: Option<ExecutionMode>,
    /// Instance count.
    #[serde(default)]
    pub instances: Option<InstanceCount>,
    /// Memory restart threshold.
    #[serde(default)]
    pub max_memory_restart: Option<String>,
    /// Auto restart flag.
    #[serde(default, alias = "autorestart")]
    pub auto_restart: Option<bool>,
    /// Watch configuration.
    #[serde(default)]
    pub watch: Option<WatchSpec>,
    /// Watch ignore list.
    #[serde(default)]
    pub ignore_watch: Option<Vec<String>>,
    /// Kill timeout in milliseconds.
    #[serde(default)]
    pub kill_timeout: Option<u64>,
    /// Kill timeout in milliseconds.
    #[serde(default)]
    pub kill_timeout_ms: Option<u64>,
    /// Min uptime in milliseconds.
    #[serde(default)]
    pub min_uptime: Option<u64>,
    /// Min uptime in milliseconds.
    #[serde(default)]
    pub min_uptime_ms: Option<u64>,
    /// Max restart count.
    #[serde(default)]
    pub max_restarts: Option<u32>,
    /// App env values.
    #[serde(default)]
    pub env: BTreeMap<String, serde_json::Value>,
    /// Stderr log file.
    #[serde(
        default,
        alias = "error",
        alias = "err",
        alias = "err_file",
        alias = "err_log"
    )]
    pub error_file: Option<PathBuf>,
    /// Stdout log file.
    #[serde(default, alias = "out", alias = "output", alias = "out_log")]
    pub out_file: Option<PathBuf>,
    /// Combined log file.
    #[serde(default, alias = "log")]
    pub log_file: Option<PathBuf>,
    /// Log date format.
    #[serde(default)]
    pub log_date_format: Option<String>,
    /// Merge logs flag.
    #[serde(default)]
    pub merge_logs: Option<bool>,
    /// Timestamp prefix flag.
    #[serde(default, alias = "time")]
    pub prefix_timestamp: Option<bool>,
    /// Cron restart expression.
    #[serde(default)]
    pub cron_restart: Option<String>,
    /// Interpreter path.
    #[serde(default, alias = "exec_interpreter")]
    pub interpreter: Option<PathBuf>,
    /// Interpreter args.
    #[serde(default, alias = "node_args", alias = "interpreterArgs")]
    pub interpreter_args: Option<StringList>,
    /// Instance variable name.
    #[serde(default)]
    pub instance_var: Option<String>,
    /// Wait ready flag.
    #[serde(default)]
    pub wait_ready: Option<bool>,
    /// Listen timeout in milliseconds.
    #[serde(default)]
    pub listen_timeout: Option<u64>,
    /// Listen timeout in milliseconds.
    #[serde(default)]
    pub listen_timeout_ms: Option<u64>,
    /// Fixed delay (ms) between exit and the next restart.
    #[serde(default)]
    pub restart_delay: Option<u64>,
    /// Initial seed (ms) for exponential backoff restart delay.
    #[serde(default)]
    pub exp_backoff_restart_delay: Option<u64>,
    /// Exit codes that mark the app stopped (no restart).
    #[serde(default)]
    pub stop_exit_codes: Option<Vec<i32>>,
    /// Extra fields, including PM2 `env_*` blocks.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Top-level config document accepted by RSPM.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigDocument {
    /// PM2 ecosystem shape.
    Apps { apps: Vec<AppConfigInput> },
    /// Single app object.
    Single(Box<AppConfigInput>),
    /// Array of app objects.
    List(Vec<AppConfigInput>),
}

impl ConfigDocument {
    /// Extracts app inputs from any supported top-level shape.
    ///
    /// ```
    /// let document = rspm_config::schema::ConfigDocument::List(Vec::new());
    /// assert!(document.into_apps().is_empty());
    /// ```
    pub fn into_apps(self) -> Vec<AppConfigInput> {
        match self {
            Self::Apps { apps } => apps,
            Self::Single(app) => vec![*app],
            Self::List(apps) => apps,
        }
    }
}
