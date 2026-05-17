//! CLI command dispatch.

pub mod describe;
pub mod env;
pub mod flush;
pub mod id;
pub mod kill;
pub mod list;
pub mod logs;
pub mod pid;
pub mod ping;
pub mod reload_logs;
pub mod reset;
pub mod resurrect;
pub mod save;
pub mod scale;
pub mod send_signal;
pub mod start;
pub mod startup;
pub mod target;

use anyhow::Result;
use rspm_client::RspmClient;

use crate::cli::Commands;

/// Runs one CLI command.
pub async fn run(command: Commands, client: &mut RspmClient) -> Result<()> {
    match command {
        Commands::Start(args) => start::run(args, client).await,
        Commands::Stop(args) => target::stop(args, client).await,
        Commands::Restart(args) => target::restart(args, client).await,
        Commands::Reload(args) => target::reload(args, client).await,
        Commands::Delete(args) => target::delete(args, client).await,
        Commands::List | Commands::Prettylist => list::run(client).await,
        Commands::Jlist => list::run_json(client).await,
        Commands::Logs(args) => logs::run(args, client).await,
        Commands::Save | Commands::Dump => save::run(client).await,
        Commands::Resurrect => resurrect::run(client).await,
        Commands::Startup(args) => startup::install(args).await,
        Commands::Unstartup(args) => startup::uninstall(args).await,
        Commands::Ping => ping::run(client).await,
        Commands::Kill => kill::run(client).await,
        Commands::SendSignal(args) => send_signal::run(args, client).await,
        Commands::Describe(args) => describe::run(args, client).await,
        Commands::Id(args) => id::run(args, client).await,
        Commands::Pid(args) => pid::run(args, client).await,
        Commands::Env(args) => env::run(args, client).await,
        Commands::Flush(args) => flush::run(args, client).await,
        Commands::Reset(args) => reset::run(args, client).await,
        Commands::ReloadLogs => reload_logs::run(client).await,
        Commands::Scale(args) => scale::run(args, client).await,
    }
}
