//! Background worker that drives periodic supervisor maintenance.
//!
//! The worker is the rspm equivalent of `pm2/lib/Worker.js`. It wakes up on a
//! fixed interval (default [`rspm_core::defaults::WORKER_INTERVAL_MS`]) and runs
//! housekeeping tasks against the [`God`] supervisor: refreshing process
//! statuses, performing memory-threshold restarts (T8.2), and triggering cron
//! restarts (T8.3).

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;
use tokio::time::{MissedTickBehavior, interval};

use crate::god::God;

/// Returns the daemon worker tick interval.
///
/// ```
/// assert_eq!(rspm_daemon::worker::tick_interval().as_millis(), 30_000);
/// ```
pub fn tick_interval() -> Duration {
    rspm_core::defaults::worker_interval()
}

/// Drives [`God::worker_tick`] on a fixed interval until `shutdown` fires.
///
/// Mirrors the `setInterval(wrappedTasks, WORKER_INTERVAL)` loop in
/// `pm2/lib/Worker.js`. Each tick is logged at debug level; errors do not stop
/// the loop, only the next interval is awaited.
pub async fn run_until(god: Arc<Mutex<God>>, mut shutdown: Receiver<()>) {
    run_until_with_interval(god, shutdown_into_signal(&mut shutdown), tick_interval()).await
}

async fn shutdown_into_signal(rx: &mut Receiver<()>) {
    let _ = rx.recv().await;
}

async fn run_until_with_interval<S>(god: Arc<Mutex<God>>, shutdown: S, period: Duration)
where
    S: std::future::Future<Output = ()>,
{
    let mut ticker = interval(period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let _ = ticker.tick().await;

    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let mut g = god.lock().await;
                if let Err(err) = g.worker_tick().await {
                    tracing::warn!(error = %err, "worker tick failed");
                }
            }
            _ = &mut shutdown => {
                tracing::debug!("worker received shutdown signal");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rspm_core::paths::RspmHome;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    #[tokio::test]
    async fn loop_stops_on_shutdown() {
        let home_dir = tempdir().expect("temp home");
        let home = RspmHome::new(home_dir.path());
        home.ensure().expect("ensure home");
        let god = Arc::new(Mutex::new(God::new(home)));
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let task = tokio::spawn(async move {
            let trip = async move {
                tokio::time::sleep(Duration::from_millis(25)).await;
            };
            run_until_with_interval(god, trip, Duration::from_millis(5)).await;
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        task.await.expect("worker task");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
