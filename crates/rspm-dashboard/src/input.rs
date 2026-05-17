//! Dashboard input placeholder.

/// Input action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputAction {
    /// Quit dashboard.
    Quit,
    /// No operation.
    Noop,
}
