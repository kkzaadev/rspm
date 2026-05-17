//! `save` command.

use anyhow::Result;
use rspm_client::RspmClient;

/// Saves the current process list.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    println!("{}", client.save().await?);
    Ok(())
}
