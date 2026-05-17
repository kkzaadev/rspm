//! JSON config loader.

use std::path::Path;

use rspm_core::Result;

use crate::schema::ConfigDocument;

/// Loads a JSON config document.
pub fn load(path: &Path) -> Result<ConfigDocument> {
    let src = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&src)?)
}
