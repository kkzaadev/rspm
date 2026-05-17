//! `scale` command — change cluster instance count.

use anyhow::Result;
use rspm_client::RspmClient;

use crate::cli::ScaleArgs;
use crate::format::table::render_process_list;

/// Resizes a cluster-mode app.
pub async fn run(args: ScaleArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.scale(args.name, args.instances).await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}
