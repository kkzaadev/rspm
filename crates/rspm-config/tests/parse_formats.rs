//! Cross-format parsing tests for `rspm-config`.
//!
//! Confirms TOML, YAML, JSON, and `ecosystem.config.js` all converge on the
//! same normalized [`rspm_core::types::AppConfig`] shape. Mirrors the canonical
//! field list from PRD §7.3 + PM2 `lib/API/schema.json`.

use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use rspm_config::{AppConfigInput, apply_defaults, env_expand::expand, load_file};
use rspm_core::types::{EnvMap, ExecutionMode, InstanceCount};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn tmp(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("rspm-cfg-{}-{n}-{name}", std::process::id()))
}

fn write_then_load(
    name: &str,
    body: &str,
) -> Result<Vec<rspm_core::types::AppConfig>, Box<dyn Error>> {
    let path = tmp(name);
    std::fs::write(&path, body)?;
    let apps = load_file(&path)?;
    std::fs::remove_file(path)?;
    Ok(apps)
}

#[test]
fn loads_yaml_apps() -> Result<(), Box<dyn Error>> {
    let apps = write_then_load(
        "apps.yaml",
        r#"
apps:
  - name: api
    script: server.js
    instances: 2
    exec_mode: cluster
"#,
    )?;
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "api");
    assert_eq!(apps[0].execution_mode, ExecutionMode::ClusterMode);
    assert_eq!(apps[0].instances, InstanceCount::Count(2));
    Ok(())
}

#[test]
fn loads_json_single_app() -> Result<(), Box<dyn Error>> {
    let apps = write_then_load(
        "app.json",
        r#"{"name":"api","script":"server.js","autorestart":false}"#,
    )?;
    assert_eq!(apps.len(), 1);
    assert!(!apps[0].auto_restart);
    Ok(())
}

#[test]
fn loads_json_apps_array() -> Result<(), Box<dyn Error>> {
    let apps = write_then_load(
        "apps.json",
        r#"[{"name":"a","script":"a.js"},{"name":"b","script":"b.js"}]"#,
    )?;
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0].name, "a");
    assert_eq!(apps[1].name, "b");
    Ok(())
}

#[test]
fn ecosystem_js_supports_instances_max_and_env_block() -> Result<(), Box<dyn Error>> {
    let apps = write_then_load(
        "ecosystem.config.js",
        r#"
module.exports = {
  apps: [{
    name: 'api',
    script: 'server.js',
    instances: 'max',
    exec_mode: 'cluster',
    env: { NODE_ENV: 'production', PORT: '3000' }
  }]
};
"#,
    )?;
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].execution_mode, ExecutionMode::ClusterMode);
    assert_eq!(apps[0].instances, InstanceCount::Named("max".into()));
    assert_eq!(apps[0].env.get("NODE_ENV"), Some(&"production".into()));
    assert_eq!(apps[0].env.get("PORT"), Some(&"3000".into()));
    Ok(())
}

#[test]
fn apply_defaults_fills_pm2_compatible_defaults() {
    let input = AppConfigInput {
        script: Some("server.js".into()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");

    assert_eq!(app.name, "server");
    assert!(app.auto_restart);
    assert_eq!(app.kill_timeout_ms, 1_600);
    assert_eq!(app.listen_timeout_ms, 8_000);
    assert_eq!(app.max_restarts, 16);
    assert_eq!(app.min_uptime_ms, 1_000);
    assert_eq!(app.restart_delay_ms, 0);
    assert!(app.exp_backoff_restart_delay_ms.is_none());
    assert!(app.stop_exit_codes.is_empty());
    assert_eq!(app.instance_var, "NODE_APP_INSTANCE");
}

#[test]
fn apply_defaults_propagates_new_restart_fields() {
    let input = AppConfigInput {
        script: Some("server.js".into()),
        restart_delay: Some(5_000),
        exp_backoff_restart_delay: Some(200),
        stop_exit_codes: Some(vec![0, 143]),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");

    assert_eq!(app.restart_delay_ms, 5_000);
    assert_eq!(app.exp_backoff_restart_delay_ms, Some(200));
    assert_eq!(app.stop_exit_codes, vec![0, 143]);
}

#[test]
fn apply_defaults_supports_kill_timeout_aliases() {
    // PM2 accepts both `kill_timeout` and the explicit `kill_timeout_ms`.
    let input = AppConfigInput {
        script: Some("server.js".into()),
        kill_timeout: Some(3_000),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.kill_timeout_ms, 3_000);
}

#[test]
fn rejects_app_input_without_script() {
    let input = AppConfigInput::default();
    let err = apply_defaults(input).expect_err("script is required");
    assert!(err.to_string().contains("script"), "msg was: {err}");
}

#[test]
fn env_expand_handles_local_env_and_missing_var() {
    let mut env = EnvMap::new();
    env.insert("PORT".into(), "3000".into());
    assert_eq!(
        expand("http://localhost:${PORT}", &env),
        "http://localhost:3000"
    );
    // Missing variables expand to the empty string (PM2 parity).
    assert_eq!(expand("/${UNSET_VAR_NAME}/x", &env), "//x");
    // Multiple substitutions in one string.
    assert_eq!(expand("${PORT}-${PORT}", &env), "3000-3000");
}
