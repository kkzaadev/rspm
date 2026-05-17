//! `logs` command.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::LogsArgs;

/// Prints log tail lines.
pub async fn run(args: LogsArgs, client: &mut RspmClient) -> Result<()> {
    let selector = args.target.as_deref().map(Selector::parse);
    for line in client.logs(selector, args.lines).await? {
        println!("{line}");
    }
    Ok(())
}
