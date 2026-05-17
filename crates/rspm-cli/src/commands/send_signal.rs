//! `sendSignal` command.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::SignalArgs;
use crate::format::table::render_process_list;

/// Sends a signal to matching processes.
pub async fn run(args: SignalArgs, client: &mut RspmClient) -> Result<()> {
    let processes = client
        .send_signal(Selector::parse(&args.target), args.signal)
        .await?;
    println!("{}", render_process_list(&processes));
    Ok(())
}
