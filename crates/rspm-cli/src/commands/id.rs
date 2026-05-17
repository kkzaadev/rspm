//! `id` command — prints pm_id(s) for a given name.

use anyhow::Result;
use rspm_client::RspmClient;

use crate::cli::TargetArgs;

/// Looks up pm_id(s) by name (or echoes back a numeric id when given one).
pub async fn run(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    if let Ok(id) = args.target.parse::<u32>() {
        println!("{id}");
        return Ok(());
    }
    let processes = client.list().await?;
    let mut printed = false;
    for info in processes.into_iter().filter(|p| p.name == args.target) {
        println!("{}", info.pm_id);
        printed = true;
    }
    if !printed {
        anyhow::bail!("process '{}' not found, try `rspm list`", args.target);
    }
    Ok(())
}
