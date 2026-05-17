//! Deploy hook placeholder.

/// Hook phase names supported by PM2 deploy.
pub fn known_hooks() -> &'static [&'static str] {
    &["pre-setup", "post-setup", "pre-deploy", "post-deploy"]
}
