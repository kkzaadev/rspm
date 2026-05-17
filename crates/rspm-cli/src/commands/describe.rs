//! `describe / show / info` command.

use anyhow::Result;
use rspm_client::RspmClient;
use rspm_protocol::Selector;

use crate::cli::TargetArgs;

/// Prints detailed info for matching processes.
pub async fn run(args: TargetArgs, client: &mut RspmClient) -> Result<()> {
    let details = client.describe(Selector::parse(&args.target)).await?;
    if details.is_empty() {
        println!("(no matching process)");
        return Ok(());
    }
    for detail in details {
        let info = &detail.info;
        println!("=== {} ({}) ===", info.name, info.pm_id);
        println!("  status              : {:?}", info.status);
        println!(
            "  pid                 : {}",
            info.pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
        println!("  script              : {}", info.script.display());
        println!("  exec_mode           : {}", detail.exec_mode);
        println!("  instances           : {}", detail.instances);
        println!("  restart_time        : {}", info.restart_time);
        println!("  unstable_restarts   : {}", info.unstable_restarts);
        println!("  auto_restart        : {}", detail.auto_restart);
        println!("  max_restarts        : {}", detail.max_restarts);
        println!("  min_uptime_ms       : {}", detail.min_uptime_ms);
        println!("  kill_timeout_ms     : {}", detail.kill_timeout_ms);
        println!("  restart_delay_ms    : {}", detail.restart_delay_ms);
        if let Some(base) = detail.exp_backoff_restart_delay_ms {
            println!("  exp_backoff_base_ms : {base}");
        }
        if let Some(limit) = detail.max_memory_restart.as_ref() {
            println!("  max_memory_restart  : {limit}");
        }
        if !detail.stop_exit_codes.is_empty() {
            println!("  stop_exit_codes     : {:?}", detail.stop_exit_codes);
        }
        println!("  watch               : {}", detail.watch);
        if let Some(path) = info.out_file.as_ref() {
            println!("  out_file            : {}", path.display());
        }
        if let Some(path) = info.error_file.as_ref() {
            println!("  error_file          : {}", path.display());
        }
        if !detail.args.is_empty() {
            println!("  args                : {:?}", detail.args);
        }
        if let Some(path) = detail.interpreter.as_ref() {
            println!("  interpreter         : {}", path.display());
        }
    }
    Ok(())
}
