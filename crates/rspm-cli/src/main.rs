//! RSPM CLI binary.

mod cli;
mod commands;
mod format;

use anyhow::Context;
use clap::Parser;
use rspm_client::RspmClient;
use rspm_core::paths::RspmHome;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let home = RspmHome::from_env().context("failed to resolve RSPM home")?;

    if cli.daemon {
        rspm_daemon::run(home).await.context("daemon failed")?;
        return Ok(());
    }

    let command = cli.command.unwrap_or_default();
    let mut client = RspmClient::connect_or_launch(home)
        .await
        .context("failed to connect to rspm daemon")?;
    commands::run(command, &mut client).await
}
