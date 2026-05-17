//! `start` command.

use anyhow::{Context, Result};
use rspm_client::RspmClient;
use rspm_config::load_file;
use rspm_core::types::{AppConfig, InstanceCount};

use crate::cli::StartArgs;
use crate::format::table::render_process_list;

/// Starts a script or config file.
pub async fn run(args: StartArgs, client: &mut RspmClient) -> Result<()> {
    let apps = if is_config_path(&args.script) {
        load_file(&args.script).with_context(|| {
            format!(
                "failed to load ecosystem/config file {}",
                args.script.display()
            )
        })?
    } else {
        vec![app_from_args(args)]
    };

    let mut processes = Vec::new();
    for app in apps {
        processes.extend(client.start_app(app).await?);
    }

    println!("{}", render_process_list(&processes));
    Ok(())
}

fn app_from_args(args: StartArgs) -> AppConfig {
    let mut app = AppConfig::from_script(args.script, args.name);
    app.args = args.args;
    app.cwd = args.cwd;
    app.interpreter = args.interpreter;
    app.auto_restart = !args.no_autorestart;
    if let Some(instances) = args.instances {
        app.instances = parse_instances(&instances);
    }
    app
}

fn parse_instances(value: &str) -> InstanceCount {
    match value.parse::<u32>() {
        Ok(count) => InstanceCount::Count(count),
        Err(_) => InstanceCount::Named(value.to_owned()),
    }
}

fn is_config_path(path: &std::path::Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    filename.ends_with(".config.js")
        || filename.ends_with(".config.cjs")
        || filename.ends_with(".config.mjs")
        || matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("json" | "yaml" | "yml" | "toml")
        )
}
