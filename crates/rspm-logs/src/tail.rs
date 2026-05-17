//! Log tail and follow helpers.
//!
//! `tail_file` returns the last N lines of a log file synchronously through
//! tokio's async fs API. `follow` returns a streaming receiver that emits new
//! lines as they are appended, reopening when the underlying inode changes
//! (rotation by [`crate::rotator::Rotator`] or external logrotate).

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::time::sleep;

use rspm_core::Result;

/// Reads the last `lines` lines from a UTF-8-ish log file.
///
/// ```
/// # async fn demo(path: &std::path::Path) -> rspm_core::Result<()> {
/// let _lines = rspm_logs::tail_file(path, 10).await?;
/// # Ok(())
/// # }
/// ```
pub async fn tail_file(path: &Path, lines: usize) -> Result<Vec<String>> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };
    let all_lines = content.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    let start = all_lines.len().saturating_sub(lines);
    Ok(all_lines.into_iter().skip(start).collect())
}

/// Default poll interval for `follow`. Matches PM2's chokidar polling for
/// missing files (250ms).
pub const DEFAULT_FOLLOW_POLL: Duration = Duration::from_millis(250);

/// Returns a stream of new lines appended to `path`. Reopens transparently
/// when the file's inode changes (rotation), so consumers do not miss output
/// across rollovers.
///
/// The initial emission is the last `lines_from_end` lines already on disk,
/// followed by anything appended after the call.
pub fn follow(path: impl Into<PathBuf>, lines_from_end: usize) -> UnboundedReceiver<String> {
    let (tx, rx) = unbounded_channel::<String>();
    let path = path.into();
    tokio::spawn(async move {
        if let Err(err) = follow_loop(&path, lines_from_end, &tx).await {
            tracing_log_warn(&path, err);
        }
    });
    rx
}

async fn follow_loop(
    path: &Path,
    lines_from_end: usize,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
) -> Result<()> {
    // Seed with the last N lines if requested.
    if lines_from_end > 0 {
        for line in tail_file(path, lines_from_end).await? {
            if tx.send(line).is_err() {
                return Ok(());
            }
        }
    }

    let mut inode = current_inode(path).await;
    let mut offset = current_size(path).await;
    let mut leftover = String::new();
    loop {
        match read_from_offset(path, offset).await {
            Ok((chunk, new_offset)) => {
                offset = new_offset;
                if !chunk.is_empty() {
                    leftover.push_str(&chunk);
                    while let Some(idx) = leftover.find('\n') {
                        let line: String = leftover.drain(..=idx).collect();
                        let trimmed = line.trim_end_matches('\n').to_owned();
                        if tx.send(trimmed).is_err() {
                            return Ok(());
                        }
                    }
                }
            }
            Err(err) => {
                tracing_log_warn(path, err);
            }
        }

        // Detect rotation by inode change; on detection, reset offset.
        let next_inode = current_inode(path).await;
        if next_inode != inode {
            inode = next_inode;
            offset = 0;
            leftover.clear();
        }

        sleep(DEFAULT_FOLLOW_POLL).await;
    }
}

async fn current_inode(path: &Path) -> Option<u64> {
    tokio::fs::metadata(path).await.ok().map(|m| m.ino())
}

async fn current_size(path: &Path) -> u64 {
    tokio::fs::metadata(path)
        .await
        .map(|m| m.len())
        .unwrap_or(0)
}

async fn read_from_offset(path: &Path, offset: u64) -> Result<(String, u64)> {
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok((String::new(), offset));
        }
        Err(err) => return Err(err.into()),
    };
    let mut reader = BufReader::new(file);
    let size = reader.get_ref().metadata().await?.len();
    if size < offset {
        // File was truncated; restart from beginning.
        return read_from_offset_fresh(reader, 0).await;
    }
    reader.seek(SeekFrom::Start(offset)).await?;
    read_from_offset_fresh(reader, offset).await
}

async fn read_from_offset_fresh(
    mut reader: BufReader<tokio::fs::File>,
    base_offset: u64,
) -> Result<(String, u64)> {
    let mut buf = String::new();
    let bytes = reader.read_to_string(&mut buf).await?;
    Ok((buf, base_offset + bytes as u64))
}

fn tracing_log_warn(path: &Path, err: impl std::fmt::Display) {
    tracing::warn!(path = %path.display(), error = %err, "follow loop error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::time::timeout;

    #[tokio::test]
    async fn streams_appended_lines() {
        let dir = tempdir().expect("temp");
        let path = dir.path().join("stream.log");
        std::fs::write(&path, b"seed-line\n").expect("seed");
        let mut rx = follow(path.clone(), 1);

        // Initial seed.
        let first = timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("recv seed");
        assert_eq!(first.as_deref(), Some("seed-line"));

        // Appended line should arrive within the poll interval.
        let appender = {
            let path = path.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(300)).await;
                tokio::fs::write(&path, b"seed-line\nappended-line\n")
                    .await
                    .expect("append");
            })
        };
        let next = timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("recv append");
        appender.await.expect("appender");
        assert_eq!(next.as_deref(), Some("appended-line"));
    }
}
