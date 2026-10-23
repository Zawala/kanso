//! `<data>/gallery/` bookkeeping: what Kanso installed, the catalog's floor,
//! and the pending record that makes a crash between a swap and a state
//! write harmless.
//!
//! `state.json` is a cache, never trusted: at start its records are
//! reconciled against the bytes on disk, and a `pending` record is resolved
//! by hashing the installed copy. `meta.json` holds the highest serial
//! accepted (even when no theme was installed), the index's digest and
//! ETags, and when the catalog was last checked.

use std::io::{self, Write as _};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::index::{Appearance, Floor, Kind};

/// One installed gallery theme.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub kind: Kind,
    pub version: u64,
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub appearance: Option<Appearance>,
    /// A color file's SHA-256, or an icon folder's `payload_digest`.
    pub digest: String,
    /// RFC 3339.
    pub installed_at: String,
}

impl From<Record> for crate::plan::Installed {
    fn from(record: Record) -> Self {
        crate::plan::Installed {
            id: record.id,
            kind: record.kind,
            version: record.version,
            name: record.name,
            family: record.family,
            digest: record.digest,
            current_digest: None,
            installed_at: record.installed_at,
        }
    }
}

/// A swap that was about to happen. Written before the rename and cleared
/// after; at start, the installed copy's digest says which side of the
/// rename a crash landed on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pending {
    pub id: String,
    pub kind: Kind,
    pub version: u64,
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub appearance: Option<Appearance>,
    /// The digest of what is about to be put in place.
    pub digest: String,
    /// RFC 3339.
    pub installed_at: String,
}

/// `state.json`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub installed: Vec<Record>,
    pub pending: Vec<Pending>,
}

impl State {
    pub fn load(path: &Path) -> State {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        write_atomic(path, &bytes)
    }

    pub fn record(&self, id: &str) -> Option<&Record> {
        self.installed.iter().find(|record| record.id == id)
    }

    /// Records an install that is already on disk.
    pub fn insert(&mut self, record: Record) {
        self.installed.retain(|other| other.id != record.id);
        self.installed.push(record);
        self.installed.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn remove(&mut self, id: &str) -> Option<Record> {
        let found = self.installed.iter().position(|record| record.id == id)?;
        Some(self.installed.remove(found))
    }

    /// Resolves a pending record after a crash. `current` is the digest of
    /// what is on disk now, or `None` when nothing is. A match means the
    /// swap completed and the new version is recorded; otherwise the old
    /// copy is still there and the pending record is only cleared. Pure.
    pub fn reconcile(&mut self, current: impl Fn(&str) -> Option<String>) {
        let pending = std::mem::take(&mut self.pending);
        for swap in pending {
            if current(&swap.id).as_deref() == Some(swap.digest.as_str()) {
                self.insert(Record {
                    id: swap.id,
                    kind: swap.kind,
                    version: swap.version,
                    name: swap.name,
                    family: swap.family,
                    appearance: swap.appearance,
                    digest: swap.digest,
                    installed_at: swap.installed_at,
                });
            }
        }
    }

    /// The installed records with what is on disk now, for [`crate::plan`].
    pub fn plan_records(
        &self,
        current: impl Fn(&Record) -> Option<String>,
    ) -> Vec<crate::plan::Installed> {
        self.installed
            .iter()
            .map(|record| crate::plan::Installed {
                id: record.id.clone(),
                kind: record.kind,
                version: record.version,
                name: record.name.clone(),
                family: record.family.clone(),
                digest: record.digest.clone(),
                current_digest: current(record),
                installed_at: record.installed_at.clone(),
            })
            .collect()
    }
}

/// `meta.json`: the catalog's floor and the last check.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Meta {
    /// The highest serial accepted.
    pub serial: u64,
    /// The digest of the exact index bytes that carried it.
    pub index_sha256: String,
    pub index_etag: Option<String>,
    pub signature_etag: Option<String>,
    /// Unix milliseconds.
    pub last_checked_ms: Option<i64>,
    /// The last failure, for the log and the quiet Settings line.
    pub last_error: Option<String>,
}

impl Meta {
    pub fn load(path: &Path) -> Meta {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        write_atomic(path, &bytes)
    }

    /// The floor a new index is checked against. Pure.
    pub fn floor(&self) -> Floor {
        Floor {
            serial: self.serial,
            digest: self.index_sha256.clone(),
        }
    }

    /// Records an index that passed every check, keeping the highest serial
    /// ever accepted. A lower serial can only arrive from a cache race, and
    /// must never lower the floor. Pure.
    pub fn accept(&mut self, serial: u64, digest: String, etag: Option<String>) {
        if serial >= self.serial {
            self.serial = serial;
            self.index_sha256 = digest;
        }
        self.index_etag = etag;
    }
}

/// The current time as RFC 3339, in UTC, without a dependency: `state.json`
/// and `plan` compare it only as text.
pub fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let days = secs / 86_400;
    let rest = secs % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        (rest % 3600) / 60,
        rest % 60
    )
}

/// Howard Hinnant's days-to-civil date conversion, for [`now_rfc3339`].
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i64::from(m <= 2), m as u32, d as u32)
}

/// Writes `bytes` to `path` through a sibling temp file, owner-only on Unix,
/// fsynced, then renamed. A failed write leaves the old file in place.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp-{}-{}",
        path.extension().and_then(|e| e.to_str()).unwrap_or("tmp"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos())
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let result = (|| {
        let mut file = options.open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        std::fs::remove_file(&tmp).ok();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips_and_reconciles_a_finished_swap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::default();
        state.pending.push(Pending {
            id: "nord".into(),
            kind: Kind::Color,
            version: 3,
            name: "Nord".into(),
            family: None,
            appearance: Some(Appearance::Dark),
            digest: "aa".into(),
            installed_at: "2026-10-23T12:00:00Z".into(),
        });
        // A crash after the rename: the new bytes are there.
        state.reconcile(|_| Some("aa".into()));
        assert_eq!(state.installed.len(), 1);
        assert_eq!(state.record("nord").unwrap().version, 3);

        // A crash before the rename: the old bytes are still there.
        let mut state = State::default();
        state.pending.push(Pending {
            id: "nord".into(),
            kind: Kind::Color,
            version: 3,
            name: "Nord".into(),
            family: None,
            appearance: None,
            digest: "bb".into(),
            installed_at: "2026-10-23T12:00:00Z".into(),
        });
        state.reconcile(|_| Some("aa".into()));
        assert!(state.installed.is_empty());

        state.save(&path).unwrap();
        assert_eq!(State::load(&path), state);
    }

    #[test]
    fn the_floor_keeps_the_highest_serial() {
        let mut meta = Meta::default();
        meta.accept(7, "aa".into(), Some("\"etag\"".into()));
        assert_eq!(
            meta.floor(),
            Floor {
                serial: 7,
                digest: "aa".into()
            }
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.json");
        meta.save(&path).unwrap();
        assert_eq!(Meta::load(&path), meta);

        // A lower serial from a stale cache race never lowers the floor.
        meta.accept(3, "bb".into(), Some("\"older\"".into()));
        assert_eq!(meta.floor().serial, 7);
        assert_eq!(meta.floor().digest, "aa");
        assert_eq!(meta.index_etag.as_deref(), Some("\"older\""));
    }

    #[test]
    fn the_date_is_rfc3339() {
        let now = now_rfc3339();
        assert_eq!(now.len(), 20);
        assert!(now.ends_with('Z'));
        assert_eq!(&now[4..5], "-");
    }
}
