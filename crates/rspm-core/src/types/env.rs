//! Environment variable map type.

use std::collections::BTreeMap;

/// Environment variables for an application.
pub type EnvMap = BTreeMap<String, String>;
