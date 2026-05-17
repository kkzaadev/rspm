//! `rspm resurrect` command.

use anyhow::Result;
use rspm_client::RspmClient;

use crate::format::table::render_process_list;

/// Re-starts every app persisted in the dump file.
pub async fn run(client: &mut RspmClient) -> Result<()> {
    let processes = client.resurrect().await?;
    if processes.is_empty() {
        println!("[rspm] dump file empty; nothing to resurrect");
    } else {
        println!("{}", render_process_list(&processes));
    }
    Ok(())
}
