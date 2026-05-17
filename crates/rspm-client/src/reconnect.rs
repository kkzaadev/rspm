//! Reconnect policy for daemon startup.

use std::time::Duration;

use tokio::time::sleep;

use rspm_core::paths::RspmHome;
use rspm_core::{Result, RspmError};

use crate::client::RspmClient;

/// Connects to the daemon with a short retry window.
pub async fn connect_with_retry(home: &RspmHome) -> Result<RspmClient> {
    let mut last_error = None;

    for _ in 0..50 {
        match RspmClient::connect(home.clone()).await {
            Ok(client) => return Ok(client),
            Err(err) => {
                last_error = Some(err);
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        RspmError::Daemon("daemon did not become ready before timeout".to_owned())
    }))
}
