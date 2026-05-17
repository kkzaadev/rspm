//! Per-app file watcher backed by [`notify`].
//!
//! Each [`AppWatcher`] owns one `notify::RecommendedWatcher` rooted at the
//! app cwd. Events that survive the include/exclude glob filters are pushed
//! onto a bounded tokio channel so the daemon can pull them in its own task.
//! Mirrors `pm2/lib/Watcher.js` (chokidar wrapper) — same shape: include
//! patterns, ignore patterns, recursive root, restart triggered on any change.

use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use rspm_core::types::WatchSpec;
use rspm_core::{Result, RspmError};

use crate::debounce::default_delay;
use crate::matcher::{build_ignores, build_includes, is_ignored};

/// File watch event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchEvent {
    /// Changed path.
    pub path: PathBuf,
}

/// Per-app watcher.
pub struct AppWatcher {
    cwd: PathBuf,
    patterns: WatchSpec,
    ignore: Vec<String>,
    debounce: Duration,
    rx: mpsc::Receiver<WatchEvent>,
    // Held to keep the underlying notify watcher alive.
    _watcher: Option<RecommendedWatcher>,
}

impl AppWatcher {
    /// Creates a watcher for an app cwd. Returns `Ok(None)` when the spec
    /// disables watching (`WatchSpec::Enabled(false)`).
    pub fn new(cwd: &Path, patterns: WatchSpec, ignore: &[String]) -> Result<Self> {
        let includes = match build_includes(&patterns)? {
            Some(includes) => includes,
            None => return Self::disabled(cwd, patterns, ignore),
        };
        let ignores = build_ignores(ignore)?;
        let cwd_owned = cwd.to_path_buf();
        let (tx, rx) = mpsc::channel::<WatchEvent>(64);

        let mut watcher: RecommendedWatcher = {
            let tx = tx.clone();
            notify::recommended_watcher(move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if !is_modify_kind(&event.kind) {
                        return;
                    }
                    for path in event.paths.iter().cloned() {
                        if includes.is_match(&path) && !is_ignored(&path, &ignores) {
                            let _ = tx.try_send(WatchEvent { path });
                        }
                    }
                }
            })
            .map_err(map_notify_err)?
        };
        watcher
            .watch(&cwd_owned, RecursiveMode::Recursive)
            .map_err(map_notify_err)?;

        Ok(Self {
            cwd: cwd_owned,
            patterns,
            ignore: ignore.to_vec(),
            debounce: default_delay(),
            rx,
            _watcher: Some(watcher),
        })
    }

    fn disabled(cwd: &Path, patterns: WatchSpec, ignore: &[String]) -> Result<Self> {
        let (_tx, rx) = mpsc::channel(1);
        Ok(Self {
            cwd: cwd.to_path_buf(),
            patterns,
            ignore: ignore.to_vec(),
            debounce: default_delay(),
            rx,
            _watcher: None,
        })
    }

    /// Sets the debounce window used by [`Self::next_event`] (defaults to
    /// [`default_delay`]).
    pub fn with_debounce(mut self, debounce: Duration) -> Self {
        self.debounce = debounce;
        self
    }

    /// Awaits the next coalesced event. Collapses bursts that occur within
    /// the debounce window into a single emission (the latest path).
    ///
    /// Returns `None` when the watcher has been disabled or all senders
    /// dropped (i.e. the watcher will never emit again).
    pub async fn next_event(&mut self) -> Option<WatchEvent> {
        self._watcher.as_ref()?;
        let first = self.rx.recv().await?;
        let mut latest = first;
        while let Ok(Some(next)) = tokio::time::timeout(self.debounce, self.rx.recv()).await {
            latest = next;
        }
        Some(latest)
    }

    /// Returns the configured ignore patterns.
    pub fn ignore(&self) -> &[String] {
        &self.ignore
    }

    /// Returns the configured watch spec.
    pub fn patterns(&self) -> &WatchSpec {
        &self.patterns
    }

    /// Returns the watch root.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Returns true when this watcher is disabled (spec was `Enabled(false)`).
    pub fn is_disabled(&self) -> bool {
        self._watcher.is_none()
    }
}

impl std::fmt::Debug for AppWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppWatcher")
            .field("cwd", &self.cwd)
            .field("patterns", &self.patterns)
            .field("ignore", &self.ignore)
            .field("debounce", &self.debounce)
            .field("disabled", &self.is_disabled())
            .finish()
    }
}

fn is_modify_kind(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}

fn map_notify_err(err: notify::Error) -> RspmError {
    RspmError::Config(format!("notify watcher: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn disabled_spec_returns_none() {
        let dir = tempdir().expect("temp");
        let mut watcher =
            AppWatcher::new(dir.path(), WatchSpec::Enabled(false), &[]).expect("new watcher");
        assert!(watcher.is_disabled());
        assert!(watcher.next_event().await.is_none());
    }

    #[tokio::test]
    async fn detects_a_change() {
        let dir = tempdir().expect("temp");
        let mut watcher = AppWatcher::new(dir.path(), WatchSpec::Enabled(true), &[])
            .expect("new watcher")
            .with_debounce(Duration::from_millis(20));

        let path = dir.path().join("hello.txt");
        let writer_path = path.clone();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            std::fs::write(&writer_path, b"hi").expect("write");
        });

        let event = tokio::time::timeout(Duration::from_secs(2), watcher.next_event())
            .await
            .expect("watcher did not emit");
        writer.await.expect("writer task");
        let event = event.expect("event");
        assert!(event.path.ends_with("hello.txt"));
    }
}
