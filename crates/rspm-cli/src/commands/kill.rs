//! `kill` command.

use anyhow::Result;
use rspm_client::RspmClient;

/// Kills the daemon.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    println!("{}", client.kill_daemon().await?);
    Ok(())
}
