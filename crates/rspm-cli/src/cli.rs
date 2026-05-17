//! Clap command definitions.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// RSPM command line interface.
#[derive(Debug, Parser)]
#[command(name = "rspm", version, about = "Rust process manager inspired by PM2")]
pub struct Cli {
    /// Run the daemon in the foreground.
    #[arg(long, hide = true)]
    pub daemon: bool,
    /// Command to run.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// CLI subcommands.
#[derive(Debug, Default, Subcommand)]
pub enum Commands {
    /// Start a script or ecosystem config.
    Start(StartArgs),
    /// Stop a process by id, name, or all.
    Stop(TargetArgs),
    /// Restart a process by id, name, or all.
    Restart(TargetArgs),
    /// Delete a process by id, name, or all.
    #[command(alias = "del")]
    Delete(TargetArgs),
    /// Reload a process by id, name, or all.
    Reload(TargetArgs),
    /// List processes.
    #[default]
    #[command(alias = "ls")]
    List,
    /// List processes as JSON.
    Jlist,
    /// List processes in a readable table.
    Prettylist,
    /// Show process logs.
    #[command(alias = "log")]
    Logs(LogsArgs),
    /// Save process list to dump file.
    Save,
    /// Alias for save.
    Dump,
    /// Reload the dump file and start its apps.
    Resurrect,
    /// Generate and install a system init script.
    Startup(StartupArgs),
    /// Disable and remove the installed init script.
    Unstartup(StartupArgs),
    /// Ping the daemon.
    Ping,
    /// Kill the daemon.
    Kill,
    /// Send a signal to a process by id, name, or all.
    SendSignal(SignalArgs),
    /// Show detailed information for a process.
    #[command(alias = "show", alias = "info")]
    Describe(TargetArgs),
    /// Print the pm_id(s) matching a name.
    Id(TargetArgs),
    /// Print the OS pid(s) of matching processes.
    Pid(OptionalTargetArgs),
    /// Print the effective env of a process.
    Env(TargetArgs),
    /// Truncate log files for one process or all.
    Flush(OptionalTargetArgs),
    /// Reset restart counters for a process.
    Reset(TargetArgs),
    /// Reopen log file descriptors (logrotate hook).
    #[command(name = "reloadLogs", alias = "reload-logs")]
    ReloadLogs,
    /// Resize a cluster-mode app to N instances.
    Scale(ScaleArgs),
}

/// `start` command arguments.
#[derive(Debug, Args)]
pub struct StartArgs {
    /// Script path or ecosystem config file.
    pub script: PathBuf,
    /// Process name.
    #[arg(short, long)]
    pub name: Option<String>,
    /// Working directory.
    #[arg(long)]
    pub cwd: Option<PathBuf>,
    /// Interpreter to use.
    #[arg(long)]
    pub interpreter: Option<PathBuf>,
    /// Number of instances.
    #[arg(short = 'i', long)]
    pub instances: Option<String>,
    /// Disable auto restart.
    #[arg(long = "no-autorestart")]
    pub no_autorestart: bool,
    /// Process arguments after `--`.
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Target command arguments.
#[derive(Debug, Args)]
pub struct TargetArgs {
    /// Process id, name, or all.
    pub target: String,
}

/// Optional target command arguments. Used by commands where omitting the
/// target means "all", e.g. `rspm flush` and `rspm pid`.
#[derive(Debug, Args)]
pub struct OptionalTargetArgs {
    /// Process id, name, or all. Defaults to all when omitted.
    pub target: Option<String>,
}

/// `scale` command arguments.
#[derive(Debug, Args)]
pub struct ScaleArgs {
    /// App name to resize.
    pub name: String,
    /// New instance count.
    pub instances: u32,
}

/// Logs command arguments.
#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Optional process id or name.
    pub target: Option<String>,
    /// Number of tail lines.
    #[arg(long, default_value_t = 100)]
    pub lines: usize,
}

/// Signal command arguments.
#[derive(Debug, Args)]
pub struct SignalArgs {
    /// Signal name, such as SIGTERM.
    pub signal: String,
    /// Process id, name, or all.
    pub target: String,
}

/// Startup command arguments.
#[derive(Debug, Args)]
pub struct StartupArgs {
    /// Override the detected init system. Accepts `systemd`, `openrc`, `sysv`.
    #[arg(long)]
    pub platform: Option<String>,
    /// User the service should run as (defaults to the current user).
    #[arg(long)]
    pub user: Option<String>,
    /// Service name override (defaults to `rspm`).
    #[arg(long, default_value = "rspm")]
    pub service: String,
}
