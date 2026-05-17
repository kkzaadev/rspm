//! Target lifecycle commands.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::TargetArgs;
use crate::format::table::render_process_list;

/// Stops a target process.
pub async fn stop(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.stop(Selector::parse(&args.target)).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}

/// Restarts a target process.
pub async fn restart(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.restart(Selector::parse(&args.target)).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}

/// Soft-reloads a target process.
pub async fn reload(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.reload(Selector::parse(&args.target)).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}

/// Deletes a target process.
pub async fn delete(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.delete(Selector::parse(&args.target)).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}
