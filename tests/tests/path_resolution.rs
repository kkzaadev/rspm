//! Mirror of `pm2/test/programmatic/path_resolution.mocha.js`.
//!
//! Validates script path resolution rules in `rspm-config::apply_defaults`:
//! absolute paths are kept as-is, relative paths are resolved against `cwd`
//! when present, and missing scripts produce a clear error.

use rspm_config::{AppConfigInput, apply_defaults};

#[test]
fn absolute_script_path_is_preserved() {
    let input = AppConfigInput {
        script: Some("/usr/bin/true".into()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.script, std::path::PathBuf::from("/usr/bin/true"));
}

#[test]
fn relative_script_is_resolved_against_cwd_when_provided() {
    let cwd = std::env::temp_dir();
    let input = AppConfigInput {
        script: Some("server.js".into()),
        cwd: Some(cwd.clone()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.script, cwd.join("server.js"));
}

#[test]
fn missing_script_field_returns_error() {
    let input = AppConfigInput::default();
    let err = apply_defaults(input).expect_err("script required");
    assert!(err.to_string().contains("script"), "msg was: {err}");
}

#[test]
fn name_is_inferred_from_script_stem_when_unset() {
    let input = AppConfigInput {
        script: Some("/srv/myapp/server.js".into()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.name, "server");
}

#[test]
fn explicit_name_overrides_inferred_one() {
    let input = AppConfigInput {
        name: Some("api".into()),
        script: Some("server.js".into()),
        ..Default::default()
    };
    let app = apply_defaults(input).expect("normalize");
    assert_eq!(app.name, "api");
}
