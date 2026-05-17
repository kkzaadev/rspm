//! Glob-based matchers for watcher include/exclude rules.
//!
//! Mirrors what `pm2/lib/Watcher.js` does with chokidar: the user supplies
//! glob patterns or a boolean to opt-in everything below the working
//! directory, plus an `ignore_watch` list. We always layer common project
//! noise (`node_modules`, `.git`, etc.) on top of the user list so the
//! default-on case doesn't kick a restart loop.

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use rspm_core::types::WatchSpec;
use rspm_core::{Result, RspmError};

/// Always-ignored patterns layered onto the user `ignore_watch` list.
pub const DEFAULT_IGNORES: &[&str] = &[
    "**/node_modules/**",
    "**/.git/**",
    "**/target/**",
    "**/logs/**",
    "**/pids/**",
    "**/dist/**",
    "**/build/**",
    "**/.next/**",
    "**/.cache/**",
];

/// Builds an include matcher from a [`WatchSpec`].
///
/// `Enabled(true)` matches everything under the watch root. `Enabled(false)`
/// returns `None` to signal the watcher should be skipped entirely. `Paths`
/// builds a glob set from the supplied patterns.
pub fn build_includes(spec: &WatchSpec) -> Result<Option<GlobSet>> {
    match spec {
        WatchSpec::Enabled(false) => Ok(None),
        WatchSpec::Enabled(true) => {
            let mut builder = GlobSetBuilder::new();
            builder.add(Glob::new("**/*").map_err(map_glob_err)?);
            Ok(Some(builder.build().map_err(map_glob_err)?))
        }
        WatchSpec::Paths(patterns) => {
            let mut builder = GlobSetBuilder::new();
            for raw in patterns {
                builder.add(Glob::new(raw).map_err(map_glob_err)?);
            }
            Ok(Some(builder.build().map_err(map_glob_err)?))
        }
    }
}

/// Builds an exclude matcher combining the configured `ignore_watch` list with
/// [`DEFAULT_IGNORES`].
pub fn build_ignores(ignore: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for raw in ignore {
        builder.add(Glob::new(raw).map_err(map_glob_err)?);
    }
    for default in DEFAULT_IGNORES {
        builder.add(Glob::new(default).map_err(map_glob_err)?);
    }
    builder.build().map_err(map_glob_err)
}

/// Returns true when `path` should be filtered out by the ignore matcher.
pub fn is_ignored(path: &Path, ignore: &GlobSet) -> bool {
    ignore.is_match(path)
}

fn map_glob_err(err: globset::Error) -> RspmError {
    RspmError::Config(format!("watch pattern: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_ignores_node_modules() {
        let ignore = build_ignores(&[]).expect("build ignores");
        assert!(ignore.is_match(PathBuf::from("/proj/node_modules/foo/index.js")));
        assert!(ignore.is_match(PathBuf::from("/proj/.git/HEAD")));
        assert!(!ignore.is_match(PathBuf::from("/proj/src/index.js")));
    }

    #[test]
    fn explicit_pattern_matches() {
        let include =
            build_includes(&WatchSpec::Paths(vec!["**/*.rs".to_owned()])).expect("includes");
        let set = include.expect("matcher returned");
        assert!(set.is_match(PathBuf::from("/proj/src/main.rs")));
        assert!(!set.is_match(PathBuf::from("/proj/src/main.go")));
    }

    #[test]
    fn enabled_false_skips_watcher() {
        let include = build_includes(&WatchSpec::Enabled(false)).expect("includes");
        assert!(include.is_none());
    }

    #[test]
    fn enabled_true_matches_everything() {
        let include = build_includes(&WatchSpec::Enabled(true)).expect("includes");
        let set = include.expect("matcher");
        assert!(set.is_match(PathBuf::from("/anything/at/all.txt")));
    }
}
