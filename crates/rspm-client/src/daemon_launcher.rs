//! Daemon launching helpers.

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};

use rspm_core::constants::RSPM_HOME_ENV;
use rspm_core::paths::RspmHome;
use rspm_core::{Result, RspmError};

/// Launches the daemon process if the socket is not accepting connections.
pub async fn launch_if_needed(home: &RspmHome) -> Result<()> {
    if rspm_ipc::IpcClient::connect(&home.rpc_socket())
        .await
        .is_ok()
    {
        return Ok(());
    }

    let daemon_bin = std::env::var_os("RSPM_DAEMON_BIN")
        .or_else(|| std::env::current_exe().ok().map(OsString::from))
        .ok_or_else(|| RspmError::Daemon("could not determine daemon executable".to_owned()))?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.daemon_log_file())?;

    let err = log.try_clone()?;
    let mut command = Command::new(daemon_bin);
    command.arg("--daemon");
    command.env(RSPM_HOME_ENV, home.root());
    command.stdin(Stdio::null());
    command.stdout(Stdio::from(log));
    command.stderr(Stdio::from(err));

    let _child = command.spawn()?;
    Ok(())
}
