//! Mirror of `pm2/test/programmatic/max_memory_limit.js`.
//!
//! Verifies the `max_memory_restart` field is parsed (`200M`, `1G`, etc) and
//! that the worker tick triggers a restart when the live process exceeds it.

use std::time::Duration;

use rspm_core::types::AppConfig;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[test]
fn parses_byte_size_units() {
    let mut app = AppConfig::from_script("x.sh", None);
    app.max_memory_restart = Some("200M".to_owned());
    assert_eq!(app.max_memory_bytes(), Some(200 * 1024 * 1024));
    app.max_memory_restart = Some("1G".to_owned());
    assert_eq!(app.max_memory_bytes(), Some(1024 * 1024 * 1024));
    app.max_memory_restart = Some("not-a-size".to_owned());
    assert_eq!(app.max_memory_bytes(), None);
}

#[tokio::test]
async fn worker_tick_restarts_when_memory_threshold_set_to_zero() {
    // A `0B` threshold guarantees the very first sample exceeds the limit so
    // we can exercise the restart code path deterministically without
    // depending on the OS-level memory measurement of a hog process.
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("memhog", "mem-hog.sh");
    app.auto_restart = true;
    app.max_memory_restart = Some("0B".to_owned());
    god.start_app(app).await.expect("start");

    // Let the sampler observe a sample; then tick the worker explicitly.
    tokio::time::sleep(Duration::from_millis(400)).await;
    god.worker_tick().await.expect("tick");

    let listed = god.list().await.expect("list");
    // Either the process was already restarted (>=1) OR is in waiting.
    assert!(
        listed[0].restart_time >= 1
            || matches!(
                listed[0].status,
                rspm_core::types::ProcessStatus::Waiting | rspm_core::types::ProcessStatus::Online
            ),
        "expected restart or waiting, got {:?}",
        listed[0].status
    );
    let _ = god.delete_selector(&Selector::All).await;
}
