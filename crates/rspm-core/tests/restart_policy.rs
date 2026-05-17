//! Unit tests for exponential backoff and restart-related defaults.
//!
//! Mirrors the behavior described in `pm2/lib/God.js:handleExit` and
//! `pm2/lib/Worker.js`: a 1.5x multiplier capped at 15s, reset after a
//! stable uptime.

use rspm_core::defaults::{
    EXP_BACKOFF_CAP_MS, EXP_BACKOFF_RESET_TIMER_MS, MAX_RESTARTS, MIN_UPTIME_MS, next_exp_backoff,
};

#[test]
fn first_backoff_uses_base_when_prev_is_zero() {
    assert_eq!(next_exp_backoff(0, 250), 250);
}

#[test]
fn first_backoff_caps_at_max() {
    assert_eq!(next_exp_backoff(0, 60_000), EXP_BACKOFF_CAP_MS);
}

#[test]
fn subsequent_backoff_multiplies_prev_by_one_point_five() {
    assert_eq!(next_exp_backoff(200, 100), 300);
    assert_eq!(next_exp_backoff(1_000, 100), 1_500);
    assert_eq!(next_exp_backoff(10_000, 100), EXP_BACKOFF_CAP_MS);
}

#[test]
fn backoff_is_monotonically_non_decreasing_until_cap() {
    let mut delay = next_exp_backoff(0, 200);
    let mut prev = delay;
    for _ in 0..20 {
        delay = next_exp_backoff(delay, 200);
        assert!(delay >= prev);
        assert!(delay <= EXP_BACKOFF_CAP_MS);
        prev = delay;
    }
    assert_eq!(prev, EXP_BACKOFF_CAP_MS);
}

#[test]
fn default_min_uptime_matches_pm2() {
    // PM2 considers a process unstable if it crashes within 1s of starting.
    assert_eq!(MIN_UPTIME_MS, 1_000);
}

#[test]
fn default_max_restarts_matches_pm2() {
    assert_eq!(MAX_RESTARTS, 16);
}

#[test]
fn reset_timer_matches_pm2_worker_interval() {
    // PM2 resets backoff once the process has been stable for one worker tick.
    assert_eq!(EXP_BACKOFF_RESET_TIMER_MS, 30_000);
}
