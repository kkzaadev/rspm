//! Config file loading.

use std::path::Path;

use rspm_core::types::AppConfig;
use rspm_core::{Result, RspmError};

use crate::format;
use crate::normalize::apply_defaults;
use crate::schema::ConfigDocument;

/// Loads and normalizes app configs from a file.
///
/// ```
/// # fn demo(path: &std::path::Path) -> rspm_core::Result<()> {
/// let _apps = rspm_config::load_file(path)?;
/// # Ok(())
/// # }
/// ```
pub fn load_file(path: &Path) -> Result<Vec<AppConfig>> {
    let document = match detect_format(path)? {
        ConfigFormat::Json => format::json::load(path)?,
        ConfigFormat::Yaml => format::yaml::load(path)?,
        ConfigFormat::Toml => format::toml::load(path)?,
        ConfigFormat::Ecosystem => format::ecosystem::load(path)?,
    };

    document
        .into_apps()
        .into_iter()
        .map(apply_defaults)
        .collect()
}

/// Parses a config document from JSON text.
pub(crate) fn parse_json_document(src: &str) -> Result<ConfigDocument> {
    Ok(serde_json::from_str(src)?)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Ecosystem,
}

fn detect_format(path: &Path) -> Result<ConfigFormat> {
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| RspmError::InvalidPath(path.to_path_buf()))?;

    if filename.ends_with(".config.js")
        || filename.ends_with(".config.cjs")
        || filename.ends_with(".config.mjs")
    {
        return Ok(ConfigFormat::Ecosystem);
    }

    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => Ok(ConfigFormat::Json),
        Some("yaml" | "yml") => Ok(ConfigFormat::Yaml),
        Some("toml") => Ok(ConfigFormat::Toml),
        Some("js" | "cjs" | "mjs") => Ok(ConfigFormat::Ecosystem),
        _ => Err(RspmError::Config(format!(
            "unsupported config file extension: {}",
            path.display()
        ))),
    }
}
