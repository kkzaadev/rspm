//! TOML config loader.

use std::path::Path;

use rspm_core::Result;

use crate::schema::ConfigDocument;

/// Loads a TOML config document.
pub fn load(path: &Path) -> Result<ConfigDocument> {
    let src = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&src)?)
}
