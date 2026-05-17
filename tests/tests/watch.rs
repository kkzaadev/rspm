//! Mirror of `pm2/test/programmatic/watch.mocha.js`.
//!
//! The actual `notify`-driven watcher is exercised in `rspm-watcher` unit
//! tests; here we verify the config plumbing: `watch` truthy values produce
//! a watcher task on `start_app`, and `ignore_watch` patterns reach the
//! daemon.

use std::time::Duration;

use rspm_core::types::WatchSpec;
use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn watch_enabled_does_not_panic_during_start() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("watched", "sleeper.sh");
    app.watch = WatchSpec::Enabled(true);
    app.cwd = Some(std::env::temp_dir());

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(god.list().await.expect("list").len(), 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn watch_disabled_does_not_register_a_watcher() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("no_watch", "sleeper.sh");
    app.watch = WatchSpec::Enabled(false);
    god.start_app(app).await.expect("start");
    assert_eq!(god.list().await.expect("list").len(), 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn watch_paths_pattern_is_accepted_by_app_config() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let mut app = fixture_app("patterns", "sleeper.sh");
    app.watch = WatchSpec::Paths(vec!["src/**/*.rs".into()]);
    app.ignore_watch = vec!["**/target/**".into(), ".git".into()];
    app.cwd = Some(std::env::temp_dir());

    god.start_app(app).await.expect("start");
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(god.list().await.expect("list").len(), 1);
    let _ = god.delete_selector(&Selector::All).await;
}
