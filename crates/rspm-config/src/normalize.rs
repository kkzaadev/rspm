//! App config normalization.

use std::collections::BTreeMap;
use std::path::PathBuf;

use rspm_core::defaults;
use rspm_core::types::{AppConfig, EnvMap, ExecutionMode, InstanceCount};
use rspm_core::{Result, RspmError};

use crate::env_expand::expand;
use crate::schema::AppConfigInput;

/// Applies defaults and validation to a raw app config.
///
/// ```
/// let mut input = rspm_config::AppConfigInput::default();
/// input.script = Some("server.js".into());
/// let app = rspm_config::apply_defaults(input).expect("valid app config");
/// assert_eq!(app.name, "server");
/// ```
pub fn apply_defaults(input: AppConfigInput) -> Result<AppConfig> {
    let script = input
        .script
        .ok_or_else(|| RspmError::Config("No script path - aborting".to_owned()))?;
    let name = input
        .name
        .unwrap_or_else(|| rspm_core::types::app::infer_name(&script));
    let cwd = input.cwd;
    let resolved_script = resolve_script(&script, cwd.as_ref())?;
    let env = normalize_env(input.env);
    let expanded_env = env
        .iter()
        .map(|(key, value)| (key.clone(), expand(value, &env)))
        .collect::<EnvMap>();
    let env_overrides = normalize_env_overrides(input.extra);

    Ok(AppConfig {
        name,
        script: resolved_script,
        args: input.args.map(|args| args.into_vec()).unwrap_or_default(),
        cwd,
        execution_mode: input.execution_mode.unwrap_or(ExecutionMode::ForkMode),
        instances: input.instances.unwrap_or(InstanceCount::Count(1)),
        max_memory_restart: input.max_memory_restart,
        auto_restart: input.auto_restart.unwrap_or_else(defaults::auto_restart),
        watch: input.watch.unwrap_or_default(),
        ignore_watch: input.ignore_watch.unwrap_or_default(),
        kill_timeout_ms: input
            .kill_timeout_ms
            .or(input.kill_timeout)
            .unwrap_or_else(defaults::kill_timeout_ms),
        min_uptime_ms: input
            .min_uptime_ms
            .or(input.min_uptime)
            .unwrap_or_else(defaults::min_uptime_ms),
        max_restarts: input.max_restarts.unwrap_or_else(defaults::max_restarts),
        env: expanded_env,
        env_overrides,
        error_file: input.error_file,
        out_file: input.out_file,
        combined_file: input.log_file,
        log_date_format: input.log_date_format,
        merge_logs: input.merge_logs.unwrap_or(false),
        prefix_timestamp: input.prefix_timestamp.unwrap_or(false),
        cron_restart: input.cron_restart,
        interpreter: input.interpreter,
        interpreter_args: input
            .interpreter_args
            .map(|args| args.into_vec())
            .unwrap_or_default(),
        instance_var: input.instance_var.unwrap_or_else(defaults::instance_var),
        wait_ready: input.wait_ready.unwrap_or(false),
        listen_timeout_ms: input
            .listen_timeout_ms
            .or(input.listen_timeout)
            .unwrap_or_else(defaults::listen_timeout_ms),
        restart_delay_ms: input
            .restart_delay
            .unwrap_or_else(defaults::restart_delay_ms),
        exp_backoff_restart_delay_ms: input.exp_backoff_restart_delay,
        stop_exit_codes: input.stop_exit_codes.unwrap_or_default(),
    })
}

fn resolve_script(script: &PathBuf, cwd: Option<&PathBuf>) -> Result<PathBuf> {
    if script.is_absolute() {
        return Ok(script.clone());
    }

    let base = match cwd {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => std::env::current_dir()?.join(path),
        None => std::env::current_dir()?,
    };

    Ok(base.join(script))
}

fn normalize_env(input: BTreeMap<String, serde_json::Value>) -> EnvMap {
    input
        .into_iter()
        .map(|(key, value)| (key, json_value_to_string(value)))
        .collect()
}

fn normalize_env_overrides(extra: BTreeMap<String, serde_json::Value>) -> BTreeMap<String, EnvMap> {
    extra
        .into_iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("env_").map(|env_name| {
                let map = value
                    .as_object()
                    .map(|object| {
                        object
                            .iter()
                            .map(|(key, value)| (key.clone(), json_value_to_string(value.clone())))
                            .collect::<EnvMap>()
                    })
                    .unwrap_or_default();
                (env_name.to_owned(), map)
            })
        })
        .collect()
}

fn json_value_to_string(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value,
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
