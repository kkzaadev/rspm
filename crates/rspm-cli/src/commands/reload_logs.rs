//! `reloadLogs` command.

use anyhow::Result;
use rspm_client::RspmClient;

/// Asks the daemon to reopen log file descriptors. Use after logrotate moved
/// the files away under it.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    let message = client.reload_logs().await?;
    println!("{message}");
    Ok(())
}
