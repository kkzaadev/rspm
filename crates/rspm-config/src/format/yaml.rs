//! YAML config loader.

use std::path::Path;

use rspm_core::Result;

use crate::schema::ConfigDocument;

/// Loads a YAML config document.
pub fn load(path: &Path) -> Result<ConfigDocument> {
    let src = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&src)?)
}
