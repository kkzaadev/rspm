//! Remote git placeholder.

/// Returns the deploy git command for documentation and tests.
pub fn pull_command() -> &'static str {
    "git pull --ff-only"
}
