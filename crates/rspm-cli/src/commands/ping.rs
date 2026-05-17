//! `ping` command.

use anyhow::Result;
use rspm_client::RspmClient;

/// Pings the daemon.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    println!("{}", client.ping().await?);
    Ok(())
}
