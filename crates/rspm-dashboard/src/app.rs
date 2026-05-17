//! Dashboard state placeholder.

/// Dashboard state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DashboardApp {
    /// Selected process index.
    pub selected: usize,
}
