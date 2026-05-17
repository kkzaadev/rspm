//! Constants copied from PM2 behavior where applicable.

/// Default ecosystem filename used by PM2.
pub const APP_CONF_DEFAULT_FILE: &str = "ecosystem.config.js";
/// RSPM home override environment variable.
pub const RSPM_HOME_ENV: &str = "RSPM_HOME";
/// Default RSPM home directory name.
pub const DEFAULT_HOME_DIR: &str = ".rspm";
/// Default daemon log filename, intentionally matching PM2.
pub const DAEMON_LOG_FILE: &str = "pm2.log";
/// Default daemon PID filename, intentionally matching PM2.
pub const DAEMON_PID_FILE: &str = "pm2.pid";
/// Request/reply socket filename.
pub const RPC_SOCKET_FILE: &str = "rpc.sock";
/// Pub/sub socket filename reserved for PM2 parity.
pub const PUB_SOCKET_FILE: &str = "pub.sock";
/// Persisted process dump filename.
pub const DUMP_FILE: &str = "dump.rspm";
/// Persisted process dump backup filename.
pub const DUMP_BACKUP_FILE: &str = "dump.rspm.bak";
/// PID directory name.
pub const PID_DIR: &str = "pids";
/// Log directory name.
pub const LOG_DIR: &str = "logs";
/// Module directory name.
pub const MODULE_DIR: &str = "modules";
/// PM2-compatible fork mode identifier.
pub const FORK_MODE_ID: &str = "fork_mode";
/// PM2-compatible cluster mode identifier.
pub const CLUSTER_MODE_ID: &str = "cluster_mode";
/// PM2-compatible online process status.
pub const ONLINE_STATUS: &str = "online";
/// PM2-compatible stopped process status.
pub const STOPPED_STATUS: &str = "stopped";
/// PM2-compatible stopping process status.
pub const STOPPING_STATUS: &str = "stopping";
/// PM2-compatible launching process status.
pub const LAUNCHING_STATUS: &str = "launching";
/// PM2-compatible errored process status.
pub const ERRORED_STATUS: &str = "errored";
/// PM2-compatible one-launch process status.
pub const ONE_LAUNCH_STATUS: &str = "one-launch-status";
/// PM2-compatible waiting restart process status.
pub const WAITING_RESTART_STATUS: &str = "waiting restart";
