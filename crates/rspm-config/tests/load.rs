use std::error::Error;

use rspm_config::load_file;

#[test]
fn loads_toml_apps() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!("rspm-config-{}-apps.toml", std::process::id()));
    std::fs::write(
        &path,
        r#"
[[apps]]
name = "api"
script = "server.js"
args = ["--port", "3000"]
autorestart = true
"#,
    )?;

    let apps = load_file(&path)?;
    std::fs::remove_file(&path)?;

    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "api");
    assert_eq!(apps[0].args, ["--port", "3000"]);
    Ok(())
}

#[test]
fn loads_ecosystem_js_apps() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!(
        "rspm-config-{}-ecosystem.config.js",
        std::process::id()
    ));
    std::fs::write(
        &path,
        "module.exports = { apps: [{ name: 'api', script: 'server.js' }] };",
    )?;

    let apps = load_file(&path)?;
    std::fs::remove_file(&path)?;

    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "api");
    Ok(())
}
