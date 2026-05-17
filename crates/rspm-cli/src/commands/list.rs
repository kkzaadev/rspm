//! `list` command.

use anyhow::Result;
use rspm_client::RspmClient;

use crate::format::table::render_process_list;

/// Prints a process table.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    let processes = client.list().await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}

/// Prints process list as JSON.
pub async fn run_json(client: &mut RspmClient) -> Result<()> {
    let processes = client.list().await?;
    println!("{}", serde_json::to_string_pretty(&processes)?);
    Ok(())
}
