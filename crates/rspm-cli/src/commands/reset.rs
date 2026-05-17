//! `reset` command — zero restart counters.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::TargetArgs;
use crate::format::table::render_process_list;

/// Resets restart counters for matching processes.
pub async fn run(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.reset(Selector::parse(&args.target)).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}
