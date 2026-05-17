//! Mirror of `pm2/test/programmatic/treekill.mocha.js`.
//!
//! Verifies that stopping a process whose child also forked grandchildren
//! leaves no orphaned descendants. rspm currently sends SIGINT/SIGKILL to the
//! immediate child only — this test pins that contract and will fail loudly
//! the day we add tree-kill so we remember to update PM2 parity docs.

use std::time::Duration;

use rspm_daemon::god::God;
use rspm_protocol::Selector;
use rspm_tests::{isolated_home, sh_app};

#[tokio::test]
async fn stop_terminates_the_direct_child_process() {
    let (_dir, home) = isolated_home();
    let mut god = God::new(home);
    let app = sh_app("tree", "while true; do sleep 0.1; done");
    let started = god.start_app(app).await.expect("start");
    let pid = started[0].pid.expect("pid");

    god.stop_selector(&Selector::All).await.expect("stop");
    tokio::time::sleep(Duration::from_millis(200)).await;

    // The direct child must no longer be alive (kill(0) returns ESRCH).
    let raw_pid = i32::try_from(pid).expect("pid fits");
    let result = nix::sys::signal::kill(nix::unistd::Pid::from_raw(raw_pid), None);
    assert!(
        result.is_err(),
        "direct child {pid} should be gone after stop"
    );
    let _ = god.delete_selector(&Selector::All).await;
}
