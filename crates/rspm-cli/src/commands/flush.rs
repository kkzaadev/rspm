//! `flush` command — truncate log files.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::OptionalTargetArgs;

/// Truncates log files for matching processes (or all when no target given).
pub async fn run(args: OptionalTargetArgs, client: &mut RspmClient) -> Result<()> {
    let selector = args
        .target
        .as_deref()
        .filter(|target| *target != "all")
        .map(Selector::parse);
    let message = client.flush(selector).await?;
    println!("{message}");
    Ok(())
}
