//! The anonymous daily count (docs/METRICS.md) on the phones: the notice
//! shown once, the features counted where they happen, and the batch sent
//! while the app is open.
//!
//! The rules live in `kanso-metrics`, as they do on the desktop. This module
//! keeps the ledger in the library's own settings (`Store::meta`), reads the
//! account and sync state from the library, and never sends before the
//! notice was seen. Development builds and tests send nothing.

use std::sync::Arc;

use kanso_core::DocType;
use kanso_import::library::SourceKind;
use kanso_metrics::{
    Active, Day, Device, Form, Id, ImportSource, Ledger, Outcome, Platform, SyncMode,
};

pub(crate) use kanso_metrics::Feature;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::{Inner, Library};

/// Whether the one-time notice was acknowledged.
const NOTICE_SEEN: &str = "mobile.metrics.notice_seen";
/// The shared client's day's bookkeeping, as JSON.
const LEDGER: &str = "mobile.metrics.ledger";

/// The anonymous usage count of one library.
#[derive(Clone)]
pub struct MobileMetrics {
    pub(crate) inner: Arc<Inner>,
}

impl Inner {
    /// The metrics handle for this library, shared by the feature calls that
    /// sit deeper than the `Library` facade.
    pub(crate) fn metrics(self: &Arc<Self>) -> MobileMetrics {
        MobileMetrics {
            inner: self.clone(),
        }
    }
}

impl Library {
    /// The anonymous usage count: the notice's state, feature counts, and
    /// the batch `send_due` sends.
    pub fn metrics(&self) -> MobileMetrics {
        self.inner.metrics()
    }
}

impl MobileMetrics {
    /// Whether the notice has been acknowledged.
    pub fn notice_seen(&self) -> Result<bool, KansoError> {
        guard(|| self.notice_seen_now())
    }

    /// Keeps that the notice was acknowledged. Counting starts after this.
    pub fn mark_notice_seen(&self) -> Result<(), KansoError> {
        guard(|| {
            self.inner.store.set_meta(NOTICE_SEEN, "1")?;
            Ok(())
        })
    }

    /// Counts `feature` once for today, if this build counts and the notice
    /// was seen. The name is one of the closed feature list; an unknown name
    /// is refused rather than counted as something else.
    pub fn record(&self, feature: &str) -> Result<(), KansoError> {
        guard(|| {
            let feature = feature_named(feature)
                .ok_or_else(|| KansoError::invalid("That isn't a feature Kanso counts."))?;
            self.record_if(enabled(), feature)
        })
    }

    /// Sends today's `app_active` and waiting feature events, if the notice
    /// was seen and this is a release build. `tablet` is the device's form,
    /// `os_version` its system version (or `unknown`), and `ai` whether AI
    /// is turned on. Says whether the Worker accepted a batch.
    pub fn send_due(&self, tablet: bool, os_version: &str, ai: bool) -> Result<bool, KansoError> {
        guard(|| self.send_due_if(enabled(), tablet, os_version, ai))
    }

    /// The internal count path, with the build gate passed in so tests can
    /// exercise the ledger without being release builds.
    pub(crate) fn record_feature(&self, feature: Feature) -> Result<(), KansoError> {
        guard(|| self.record_if(enabled(), feature))
    }

    /// Counts a feature from product code. Metrics are best effort: a write
    /// failure must never turn a successful note, lock, share or import into
    /// an error.
    pub(crate) fn count(&self, feature: Feature) {
        let _ = self.record_feature(feature);
    }

    fn record_if(&self, enabled: bool, feature: Feature) -> Result<(), KansoError> {
        if !enabled || !self.notice_seen_now()? {
            return Ok(());
        }
        let now = Day::today();
        let _gate = crate::boundary::held(&self.inner.metrics_gate);
        let mut ledger = self.read_ledger()?;
        if ledger.record(now, feature) {
            self.write_ledger(&ledger)?;
        }
        Ok(())
    }

    fn send_due_if(
        &self,
        enabled: bool,
        tablet: bool,
        os_version: &str,
        ai: bool,
    ) -> Result<bool, KansoError> {
        if !enabled || !self.notice_seen_now()? {
            return Ok(false);
        }
        let Some(platform) = Platform::current() else {
            return Ok(false);
        };
        let today = Day::today();
        let form = if tablet { Form::Tablet } else { Form::Phone };
        let device = Device::new(platform, form, env!("CARGO_PKG_VERSION"));
        let notes = self
            .inner
            .store
            .counts()?
            .get(&DocType::Note)
            .copied()
            .unwrap_or(0);
        let active = Active::new(
            os_version,
            self.sync_mode()?,
            ai,
            u64::try_from(notes).unwrap_or(u64::MAX),
        );
        // Read the ledger and choose the batch under the gate, then release
        // it while the request is on the wire so a feature used meanwhile is
        // not overwritten by this send's older copy.
        let batch = {
            let _gate = crate::boundary::held(&self.inner.metrics_gate);
            let mut ledger = self.read_ledger()?;
            let id = self.id(&mut ledger);
            let Some(batch) = ledger.due(today, &id, &device, &active) else {
                // The install ID may have just been made; keep it for next time.
                self.write_ledger(&ledger)?;
                return Ok(false);
            };
            // Keep a newly made install ID even if the send then fails.
            self.write_ledger(&ledger)?;
            batch
        };
        let outcome = kanso_metrics::send(&batch, enabled);
        let _gate = crate::boundary::held(&self.inner.metrics_gate);
        let mut ledger = self.read_ledger()?;
        ledger.settle(&batch, outcome);
        self.write_ledger(&ledger)?;
        Ok(outcome == Outcome::Accepted)
    }

    fn notice_seen_now(&self) -> Result<bool, KansoError> {
        Ok(self.inner.store.meta(NOTICE_SEEN)?.as_deref() == Some("1"))
    }

    fn read_ledger(&self) -> Result<Ledger, KansoError> {
        let Some(text) = self.inner.store.meta(LEDGER)? else {
            return Ok(Ledger::default());
        };
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    fn write_ledger(&self, ledger: &Ledger) -> Result<(), KansoError> {
        let text =
            serde_json::to_string(ledger).map_err(|err| KansoError::storage(err.to_string()))?;
        self.inner.store.set_meta(LEDGER, &text)?;
        Ok(())
    }

    /// The account's ID while signed in and holding its data key, else this
    /// install's. A new install ID is kept in `ledger` for the caller to
    /// save.
    fn id(&self, ledger: &mut Ledger) -> Id {
        let account = crate::syncing::SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()
        .ok()
        .flatten()
        .and_then(|session| Id::parse(&session.metrics_id()));
        if let Some(id) = account {
            return id;
        }
        if let Some(id) = ledger.existing_install_id() {
            return id.clone();
        }
        ledger.install_id()
    }

    /// `off` unless signed in with sync on; `kanso` on Kanso's own server,
    /// else `self_hosted`.
    fn sync_mode(&self) -> Result<SyncMode, KansoError> {
        let settings = crate::sync_words::read_settings(&self.inner.store)?;
        if !settings.enabled {
            return Ok(SyncMode::Off);
        }
        let session = crate::syncing::SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()?;
        let Some(session) = session else {
            return Ok(SyncMode::Off);
        };
        Ok(if kanso_sync::is_owner_server(&session.origin) {
            SyncMode::Kanso
        } else {
            SyncMode::SelfHosted
        })
    }
}

/// Whether this executable may count: a release build, never a test or a
/// development build. The Dart side also only starts the notice in the
/// production environment, so a `flutter run --release` of the development
/// entry point still counts nothing (it never sees the notice).
fn enabled() -> bool {
    !cfg!(debug_assertions) && !cfg!(test)
}

/// The closed feature names the bridge and the app's own calls use.
fn feature_named(name: &str) -> Option<Feature> {
    Some(match name {
        "note_created" => Feature::NoteCreated,
        "voice_input" => Feature::VoiceInput,
        "publish" => Feature::Publish,
        "lock" => Feature::Lock,
        "ai" => Feature::Ai,
        "quick_capture" => Feature::QuickCapture,
        "shared_note" => Feature::SharedNote,
        "export" => Feature::Export,
        _ => return None,
    })
}

/// The metrics `source` an import kind maps to. The kinds the published list
/// doesn't name are `other`.
pub(crate) fn import_source(kind: SourceKind) -> ImportSource {
    match kind {
        SourceKind::Markdown => ImportSource::Markdown,
        SourceKind::Obsidian => ImportSource::Obsidian,
        SourceKind::Logseq => ImportSource::Logseq,
        SourceKind::Notion => ImportSource::Notion,
        SourceKind::Evernote => ImportSource::Evernote,
        SourceKind::Bear => ImportSource::Bear,
        SourceKind::Joplin => ImportSource::Joplin,
        SourceKind::StandardNotes => ImportSource::StandardNotes,
        SourceKind::Simplenote => ImportSource::Simplenote,
        SourceKind::UpNote => ImportSource::Upnote,
        SourceKind::AppleNotes => ImportSource::AppleNotes,
        SourceKind::AppleJournal
        | SourceKind::GoogleKeep
        | SourceKind::Html
        | SourceKind::Roam
        | SourceKind::TextBundle
        | SourceKind::Tomboy => ImportSource::Other,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_sync::{MemorySecrets, MemoryServer};

    use super::*;

    fn library() -> (tempfile::TempDir, Library) {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        (dir, library)
    }

    #[test]
    fn names_are_the_closed_feature_list() {
        for (name, feature) in [
            ("note_created", Feature::NoteCreated),
            ("voice_input", Feature::VoiceInput),
            ("publish", Feature::Publish),
            ("lock", Feature::Lock),
            ("ai", Feature::Ai),
            ("quick_capture", Feature::QuickCapture),
            ("shared_note", Feature::SharedNote),
            ("export", Feature::Export),
        ] {
            assert_eq!(feature_named(name), Some(feature));
        }
        for name in ["import", "", "telemetry", "Note_Created"] {
            assert_eq!(feature_named(name), None, "{name}");
        }
    }

    #[test]
    fn import_sources_are_closed() {
        assert_eq!(import_source(SourceKind::Obsidian), ImportSource::Obsidian);
        assert_eq!(
            import_source(SourceKind::AppleNotes),
            ImportSource::AppleNotes
        );
        assert_eq!(import_source(SourceKind::Html), ImportSource::Other);
        assert_eq!(import_source(SourceKind::Tomboy), ImportSource::Other);
    }

    #[test]
    fn notice_and_ledger_are_kept_in_the_library() {
        let (_dir, library) = library();
        let metrics = library.metrics();
        assert!(!metrics.notice_seen().unwrap());
        metrics.mark_notice_seen().unwrap();
        assert!(metrics.notice_seen().unwrap());
        assert!(!metrics.record_if(enabled(), Feature::Lock).is_err());
        metrics.record_if(true, Feature::Lock).unwrap();
        let ledger = metrics.read_ledger().unwrap();
        assert!(
            ledger.existing_install_id().is_none(),
            "no ID made before a send"
        );
        let legacy = library.metrics();
        assert!(legacy.notice_seen().unwrap(), "the same library keeps it");
    }

    #[test]
    fn a_damaged_ledger_costs_a_count_not_the_library() {
        let (_dir, library) = library();
        library.inner.store.set_meta(LEDGER, "{not json").unwrap();
        let metrics = library.metrics();
        assert_eq!(metrics.read_ledger().unwrap(), Ledger::default());
        metrics.write_ledger(&Ledger::default()).unwrap();
        assert_eq!(metrics.read_ledger().unwrap(), Ledger::default());
    }
}
