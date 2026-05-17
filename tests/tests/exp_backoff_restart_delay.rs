//! Mirror of `pm2/test/programmatic/exp_backoff_restart_delay.mocha.js`.
//!
//! Validates the exponential backoff algorithm: first restart uses the base
//! delay, subsequent restarts multiply by 1.5x (capped at
//! `EXP_BACKOFF_CAP_MS`), and stable uptime resets the counter.

use rspm_core::defaults::{EXP_BACKOFF_CAP_MS, next_exp_backoff};

#[test]
fn should_set_exponential_backoff_restart_to_base_on_first_attempt() {
    assert_eq!(next_exp_backoff(0, 100), 100);
}

#[test]
fn should_have_incremented_the_prev_restart_delay_after_each_attempt() {
    let base = 100;
    let mut delay = next_exp_backoff(0, base);
    assert_eq!(delay, 100);
    delay = next_exp_backoff(delay, base);
    assert_eq!(delay, 150);
    delay = next_exp_backoff(delay, base);
    assert_eq!(delay, 225);
    delay = next_exp_backoff(delay, base);
    assert_eq!(delay, 337); // floor((225 * 3) / 2)
}

#[test]
fn should_cap_at_exp_backoff_cap_ms() {
    let mut delay = 12_000;
    for _ in 0..20 {
        delay = next_exp_backoff(delay, 100);
    }
    assert_eq!(delay, EXP_BACKOFF_CAP_MS);
}

#[test]
fn should_reset_prev_restart_delay_when_caller_passes_zero() {
    // Caller-side reset: when the worker tick clears prev to 0, the next
    // computation should fall back to the base again.
    let mut delay = 10_000;
    for _ in 0..5 {
        delay = next_exp_backoff(delay, 100);
    }
    let after_reset = next_exp_backoff(0, 100);
    assert_eq!(after_reset, 100);
}
