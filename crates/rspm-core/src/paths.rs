//! RSPM home path resolution.

use std::env;
use std::path::{Path, PathBuf};

use crate::constants::{
    DAEMON_LOG_FILE, DAEMON_PID_FILE, DEFAULT_HOME_DIR, DUMP_BACKUP_FILE, DUMP_FILE, LOG_DIR,
    MODULE_DIR, PID_DIR, PUB_SOCKET_FILE, RPC_SOCKET_FILE, RSPM_HOME_ENV,
};
use crate::error::{Result, RspmError};

/// Resolved RSPM home and its PM2-compatible child paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RspmHome {
    root: PathBuf,
}

impl RspmHome {
    /// Creates a home value from a path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.root().ends_with("rspm"));
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { root: path.into() }
    }

    /// Resolves `$RSPM_HOME` or defaults to `~/.rspm`.
    ///
    /// ```
    /// let resolved = rspm_core::paths::RspmHome::from_env();
    /// assert!(resolved.is_ok());
    /// ```
    pub fn from_env() -> Result<Self> {
        if let Some(value) = env::var_os(RSPM_HOME_ENV) {
            return Ok(Self::new(PathBuf::from(value)));
        }

        if let Some(home) = env::var_os("HOME") {
            return Ok(Self::new(PathBuf::from(home).join(DEFAULT_HOME_DIR)));
        }

        Ok(Self::new(PathBuf::from("/etc").join(DEFAULT_HOME_DIR)))
    }

    /// Returns the root directory.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert_eq!(home.root(), std::path::Path::new("/tmp/rspm"));
    /// ```
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Creates the home directory structure expected by the daemon and client.
    ///
    /// ```
    /// # let dir = std::env::temp_dir().join("rspm-doc-ensure");
    /// let home = rspm_core::paths::RspmHome::new(&dir);
    /// let _ = home.ensure();
    /// ```
    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.log_dir())?;
        std::fs::create_dir_all(self.pid_dir())?;
        std::fs::create_dir_all(self.module_dir())?;
        Ok(())
    }

    /// Returns the request/reply socket path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.rpc_socket().ends_with("rpc.sock"));
    /// ```
    pub fn rpc_socket(&self) -> PathBuf {
        self.root.join(RPC_SOCKET_FILE)
    }

    /// Returns the pub/sub socket path reserved for event parity.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.pub_socket().ends_with("pub.sock"));
    /// ```
    pub fn pub_socket(&self) -> PathBuf {
        self.root.join(PUB_SOCKET_FILE)
    }

    /// Returns the process dump path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.dump_file().ends_with("dump.rspm"));
    /// ```
    pub fn dump_file(&self) -> PathBuf {
        self.root.join(DUMP_FILE)
    }

    /// Returns the process dump backup path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.dump_backup_file().ends_with("dump.rspm.bak"));
    /// ```
    pub fn dump_backup_file(&self) -> PathBuf {
        self.root.join(DUMP_BACKUP_FILE)
    }

    /// Returns the log directory path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.log_dir().ends_with("logs"));
    /// ```
    pub fn log_dir(&self) -> PathBuf {
        self.root.join(LOG_DIR)
    }

    /// Returns the PID directory path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.pid_dir().ends_with("pids"));
    /// ```
    pub fn pid_dir(&self) -> PathBuf {
        self.root.join(PID_DIR)
    }

    /// Returns the module directory path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.module_dir().ends_with("modules"));
    /// ```
    pub fn module_dir(&self) -> PathBuf {
        self.root.join(MODULE_DIR)
    }

    /// Returns the daemon log file path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.daemon_log_file().ends_with("pm2.log"));
    /// ```
    pub fn daemon_log_file(&self) -> PathBuf {
        self.root.join(DAEMON_LOG_FILE)
    }

    /// Returns the daemon PID file path.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.daemon_pid_file().ends_with("pm2.pid"));
    /// ```
    pub fn daemon_pid_file(&self) -> PathBuf {
        self.root.join(DAEMON_PID_FILE)
    }

    /// Returns a path inside the log directory for an app stream.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.app_log_file("api", "out").ends_with("api-out.log"));
    /// ```
    pub fn app_log_file(&self, app_name: &str, stream: &str) -> PathBuf {
        self.log_dir()
            .join(format!("{}-{}.log", sanitize_name(app_name), stream))
    }

    /// Returns a path inside the PID directory for an app process.
    ///
    /// ```
    /// let home = rspm_core::paths::RspmHome::new("/tmp/rspm");
    /// assert!(home.app_pid_file("api", 0).ends_with("api-0.pid"));
    /// ```
    pub fn app_pid_file(&self, app_name: &str, pm_id: u32) -> PathBuf {
        self.pid_dir()
            .join(format!("{}-{}.pid", sanitize_name(app_name), pm_id))
    }
}

/// Converts a PM2 app name to a filesystem-safe basename.
///
/// ```
/// assert_eq!(rspm_core::paths::sanitize_name("my app"), "my-app");
/// ```
pub fn sanitize_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
            result.push(ch);
        } else {
            result.push('-');
        }
    }

    if result.is_empty() {
        "app".to_owned()
    } else {
        result
    }
}

/// Converts a path to a string for process environments.
///
/// ```
/// let s = rspm_core::paths::path_to_string(std::path::Path::new("/tmp"));
/// assert_eq!(s.expect("valid utf-8 path"), "/tmp");
/// ```
pub fn path_to_string(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| RspmError::InvalidPath(path.to_path_buf()))
}
