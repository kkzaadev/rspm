//! Mirror of `pm2/test/programmatic/env_switching.js`.
//!
//! Validates how the rspm-config layer materializes the user's `env` block
//! and PM2-style `env_<environment>` overrides. The actual runtime switch
//! (`--env production`) is performed by the CLI layer; here we verify that
//! both shapes are captured into `AppConfig.env` / `env_overrides`.

use rspm_config::{AppConfigInput, apply_defaults};
use serde_json::json;

#[test]
fn env_block_is_preserved_after_normalization() {
    let mut env = std::collections::BTreeMap::new();
    env.insert("PORT".to_string(), json!("3000"));
    env.insert("NODE_ENV".to_string(), json!("development"));

    let input = AppConfigInput {
        script: Some("server.js".into()),
        env,
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.env.get("PORT"), Some(&"3000".to_string()));
    assert_eq!(app.env.get("NODE_ENV"), Some(&"development".to_string()));
}

#[test]
fn env_override_block_is_captured_in_env_overrides() {
    let mut extra = std::collections::BTreeMap::new();
    extra.insert(
        "env_production".to_string(),
        json!({ "NODE_ENV": "production", "DATABASE_URL": "postgres://..." }),
    );

    let input = AppConfigInput {
        script: Some("server.js".into()),
        extra,
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    let production = app
        .env_overrides
        .get("production")
        .expect("production override present");
    assert_eq!(production.get("NODE_ENV"), Some(&"production".to_string()));
    assert!(production.contains_key("DATABASE_URL"));
}

#[test]
fn env_variable_expansion_resolves_dollar_curly_references() {
    let mut env = rspm_core::types::EnvMap::new();
    env.insert("PORT".into(), "3000".into());
    assert_eq!(
        rspm_config::env_expand::expand("listen on :${PORT}", &env),
        "listen on :3000"
    );
}

#[test]
fn env_variable_expansion_handles_missing_values_as_empty() {
    let env = rspm_core::types::EnvMap::new();
    assert_eq!(
        rspm_config::env_expand::expand("/${MISSING_VAR}/path", &env),
        "//path"
    );
}
