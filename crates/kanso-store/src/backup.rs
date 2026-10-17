//! Periodic copies of the database made with `VACUUM INTO`.

use std::path::{Path, PathBuf};

use crate::{Result, Store};

/// Where backups go, how many to keep, and how often to make one.
#[derive(Clone, Debug)]
pub struct BackupPolicy {
    pub dir: PathBuf,
    pub keep: usize,
    pub min_interval_ms: i64,
}

impl BackupPolicy {
    /// One backup a day, two weeks kept.
    pub fn daily(dir: impl Into<PathBuf>) -> Self {
        BackupPolicy {
            dir: dir.into(),
            keep: 14,
            min_interval_ms: 24 * 60 * 60 * 1000,
        }
    }
}

const PREFIX: &str = "kanso-";
const SUFFIX: &str = ".db";

fn backups(dir: &Path) -> Result<Vec<(i64, PathBuf)>> {
    let mut found = Vec::new();
    if !dir.exists() {
        return Ok(found);
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let stamp = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_prefix(PREFIX))
            .and_then(|n| n.strip_suffix(SUFFIX))
            .and_then(|n| n.parse::<i64>().ok());
        if let Some(stamp) = stamp {
            found.push((stamp, path));
        }
    }
    found.sort();
    Ok(found)
}

impl Store {
    /// Writes a consistent copy of the database to `path`.
    pub fn backup_to(&self, path: &Path) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        let target = path.to_string_lossy().into_owned();
        self.read(|conn| {
            conn.execute("VACUUM INTO ?", [target])?;
            Ok(())
        })
    }

    /// Makes a backup when the newest one is older than the policy's interval,
    /// then deletes the oldest beyond `keep`. Returns the new backup's path.
    pub fn backup_if_due(&self, policy: &BackupPolicy, now_ms: i64) -> Result<Option<PathBuf>> {
        std::fs::create_dir_all(&policy.dir)?;
        let existing = backups(&policy.dir)?;
        if existing
            .last()
            .is_some_and(|(stamp, _)| now_ms - stamp < policy.min_interval_ms)
        {
            return Ok(None);
        }
        let path = policy.dir.join(format!("{PREFIX}{now_ms}{SUFFIX}"));
        self.backup_to(&path)?;
        let all = backups(&policy.dir)?;
        let excess = all.len().saturating_sub(policy.keep.max(1));
        for (_, old) in all.into_iter().take(excess) {
            std::fs::remove_file(old)?;
        }
        Ok(Some(path))
    }
}
