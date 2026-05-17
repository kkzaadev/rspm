//! Daemon PID file helpers.

use std::io::Write;

use rspm_core::Result;
use rspm_core::paths::RspmHome;

/// Writes the current process ID to the daemon PID file.
pub fn write_pid(home: &RspmHome) -> Result<()> {
    let path = home.daemon_pid_file();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(path)?;
    write!(file, "{}", std::process::id())?;
    Ok(())
}
