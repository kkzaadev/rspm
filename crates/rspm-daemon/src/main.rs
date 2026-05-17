//! Standalone daemon binary.

use rspm_core::paths::RspmHome;

#[tokio::main]
async fn main() -> rspm_core::Result<()> {
    let home = RspmHome::from_env()?;
    rspm_daemon::run(home).await
}
