//! Mirror of `pm2/test/programmatic/common.mocha.js`.
//!
//! Common helpers in PM2 normalize app configuration (`prepareAppConf`) and
//! provide utility predicates. The rspm equivalents live in
//! `rspm_config::apply_defaults` and `rspm_core::defaults`.

use rspm_config::{AppConfigInput, apply_defaults};
use rspm_core::defaults;

#[test]
fn defaults_match_pm2_constants() {
    assert_eq!(defaults::KILL_TIMEOUT_MS, 1_600);
    assert_eq!(defaults::WORKER_INTERVAL_MS, 30_000);
    assert_eq!(defaults::LISTEN_TIMEOUT_MS, 8_000);
    assert_eq!(defaults::MIN_UPTIME_MS, 1_000);
    assert_eq!(defaults::MAX_RESTARTS, 16);
    assert_eq!(defaults::DEFAULT_INSTANCE_VAR, "NODE_APP_INSTANCE");
}

#[test]
fn prepare_app_conf_applies_pm2_defaults() {
    let input = AppConfigInput {
        script: Some("server.js".into()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert!(app.auto_restart);
    assert_eq!(app.kill_timeout_ms, defaults::KILL_TIMEOUT_MS);
    assert_eq!(app.listen_timeout_ms, defaults::LISTEN_TIMEOUT_MS);
    assert_eq!(app.max_restarts, defaults::MAX_RESTARTS);
    assert_eq!(app.min_uptime_ms, defaults::MIN_UPTIME_MS);
    assert_eq!(app.instance_var, defaults::DEFAULT_INSTANCE_VAR);
}

#[test]
fn kill_timeout_alias_is_accepted() {
    let input = AppConfigInput {
        script: Some("server.js".into()),
        kill_timeout: Some(2_500),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.kill_timeout_ms, 2_500);
}

#[test]
fn autorestart_alias_is_accepted() {
    let input = AppConfigInput {
        script: Some("server.js".into()),
        auto_restart: Some(false),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert!(!app.auto_restart);
}
