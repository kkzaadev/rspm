//! Mirror of `pm2/test/programmatic/signals.js`.
//!
//! Verifies that `send_signal` accepts PM2-style names and that invalid
//! signal names produce a clear error.

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn send_signal_accepts_pm2_style_names() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("sig", "sleeper.sh"))
        .await
        .expect("start");

    // Every alias should succeed when the process exists.
    for name in ["SIGUSR1", "USR1", "SIGHUP", "HUP", "SIGTERM"] {
        let result = god.send_signal(&Selector::All, name).await;
        assert!(
            result.is_ok(),
            "signal {name} should succeed, got: {:?}",
            result.err()
        );
    }
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn send_signal_rejects_unknown_signal_name() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("sig_bad", "sleeper.sh"))
        .await
        .expect("start");

    let err = god
        .send_signal(&Selector::All, "SIGNOPE")
        .await
        .expect_err("unknown signal");
    assert!(err.to_string().contains("NOPE"), "msg was {err}");
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn send_signal_to_missing_process_returns_not_found() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let err = god
        .send_signal(&Selector::Id(99), "SIGUSR1")
        .await
        .expect_err("missing");
    assert!(err.to_string().contains("99"), "msg was {err}");
}
