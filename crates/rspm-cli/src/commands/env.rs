//! `env` command — prints effective env for a process.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::TargetArgs;

/// Prints the effective env map(s) for matching processes.
pub async fn run(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let envs = client.env_for(Selector::parse(&args.target)).await?;
    if envs.is_empty() {
        println!("(no matching process)");
        return Ok(());
    }
    for (pm_id, env) in envs {
        println!("# pm_id={pm_id}");
        for (key, value) in env {
            println!("{key}={value}");
        }
    }
    Ok(())
}
