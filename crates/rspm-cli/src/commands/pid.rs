//! `pid` command — prints OS pid(s) for matching processes.

use anyhow::Result;
use rspm_client::RspmClient;

use crate::cli::OptionalTargetArgs;

/// Prints OS pid(s) for matching processes.
pub async fn run(args: OptionalTargetArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client.list().await?;
    let matches: Vec<_> = match args.target.as_deref() {
        None | Some("all") => processes.iter().collect(),
        Some(target) => match target.parse::<u32>() {
            Ok(id) => processes.iter().filter(|p| p.pm_id == id).collect(),
            Err(_) => processes.iter().filter(|p| p.name == target).collect(),
        },
    };
    if matches.is_empty() {
        anyhow::bail!(
            "no matching process for '{}', try `rspm list`",
            args.target.as_deref().unwrap_or("all")
        );
    }
    for info in matches {
        match info.pid {
            Some(pid) => println!("{pid}"),
            None => println!("0"),
        }
    }
    Ok(())
}
