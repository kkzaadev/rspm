//! Module registry placeholder.

/// Installed module record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleRecord {
    /// Module name.
    pub name: String,
}

/// Returns an empty module registry.
pub fn list() -> Vec<ModuleRecord> {
    Vec::new()
}
