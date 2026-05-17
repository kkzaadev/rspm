//! Mirror of `pm2/test/programmatic/resurect_state.mocha.js`.
//!
//! Note: filename keeps the original PM2 typo (`resurect`) for one-to-one
//! cross-reference with the upstream Mocha file.

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn resurrect_skips_apps_already_running() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home.clone());

    let mut app = fixture_app("dup", "sleeper.sh");
    app.name = "stay".into();
    god.start_app(app).await.expect("start");
    god.save().await.expect("save");

    // Resurrect while the same app is still running — must not double-spawn.
    let started = god.resurrect().await.expect("resurrect");
    assert!(
        started.is_empty(),
        "no fresh starts when app already running"
    );

    let listed = god.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn dump_then_resurrect_preserves_app_name() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home.clone());

    let mut app = fixture_app("named", "sleeper.sh");
    app.name = "preserved".into();
    god.start_app(app).await.expect("start");
    god.save().await.expect("save");

    god.delete_selector(&Selector::All).await.expect("delete");
    drop(god);

    let mut fresh = God::new(home);
    let started = fresh.resurrect().await.expect("resurrect");
    assert_eq!(started.len(), 1);
    assert_eq!(started[0].name, "preserved");
    let _ = fresh.delete_selector(&Selector::All).await;
}
