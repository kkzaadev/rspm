//! Mirror of `pm2/test/programmatic/id.mocha.js`.
//!
//! Verifies the daemon assigns monotonic `pm_id` values starting from `0` and
//! resets to `0` only when the registry is fully emptied.

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{fixture_app, isolated_home};

#[tokio::test]
async fn pm_id_starts_at_zero_and_increments_per_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let first = god
        .start_app(fixture_app("p", "sleeper.sh"))
        .await
        .expect("start");
    let second = god
        .start_app(fixture_app("q", "sleeper.sh"))
        .await
        .expect("start");
    assert_eq!(first[0].pm_id, 0);
    assert_eq!(second[0].pm_id, 1);
    let _ = god.delete_selector(&Selector::All).await;
}

#[tokio::test]
async fn pm_id_resets_after_delete_all() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    god.start_app(fixture_app("first", "sleeper.sh"))
        .await
        .expect("start");
    god.start_app(fixture_app("second", "sleeper.sh"))
        .await
        .expect("start");
    god.delete_selector(&Selector::All).await.expect("delete");

    let restarted = god
        .start_app(fixture_app("third", "sleeper.sh"))
        .await
        .expect("start");
    assert_eq!(
        restarted[0].pm_id, 0,
        "registry is empty -> next_id resets to 0"
    );
    let _ = god.delete_selector(&Selector::All).await;
}
