//! The batch as the Worker reads it (docs/METRICS.md › The batch): version
//! 1, one ID, 1 to 32 events, each with exactly its props.

use serde::{Serialize, Serializer};

use crate::values::{
    Active, AttachmentsMb, Channel, CountRange, Day, Device, EditorMode, Feature, Form, Id,
    ImportSource, LocaleLang, NotebooksRange, NotesRange, Platform, PluginsRange, SessionLength,
    SyncErrors, SyncMode, Theme, ThemeMode,
};

/// The most events a batch carries.
pub const MAX_EVENTS: usize = 32;
/// The most bytes a batch's body may be.
pub const MAX_BYTES: usize = 16 * 1024;

/// What happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Once a day per ID.
    AppActive(Active),
    /// Once per launch.
    AppOpened,
    /// A session ended (quit or background).
    Session(SessionLength),
    /// A feature used this many times since the last batch.
    FeatureUsed(Feature, u32),
}

/// One event, on the UTC day it happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub day: Day,
    pub kind: EventKind,
}

/// What one request sends: who is counted, the device every event shares,
/// and the events. Made by [`crate::Ledger::due`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub id: Id,
    pub device: Device,
    pub events: Vec<Event>,
}

impl Batch {
    /// The request's body.
    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("a batch is plain JSON")
    }
}

#[derive(Serialize)]
struct Wire<'a> {
    v: u8,
    id: &'a str,
    events: Vec<WireEvent<'a>>,
}

#[derive(Serialize)]
struct WireEvent<'a> {
    event: &'static str,
    day: Day,
    props: WireProps<'a>,
}

/// Every prop any event may carry, in the page's order; an absent one is
/// left out.
#[derive(Serialize)]
struct WireProps<'a> {
    platform: Platform,
    form: Form,
    app_version: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    os_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sync: Option<SyncMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ai: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<NotesRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locale_lang: Option<&'a LocaleLang>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theme_mode: Option<ThemeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theme: Option<&'a Theme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    editor_mode: Option<EditorMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugins: Option<PluginsRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notebooks: Option<NotebooksRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments_mb: Option<AttachmentsMb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_errors: Option<SyncErrors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<Channel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feature: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<ImportSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<CountRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<SessionLength>,
}

impl<'a> WireProps<'a> {
    fn base(device: &'a Device) -> WireProps<'a> {
        WireProps {
            platform: device.platform(),
            form: device.form(),
            app_version: device.app_version(),
            os_version: None,
            sync: None,
            ai: None,
            notes: None,
            locale_lang: None,
            theme_mode: None,
            theme: None,
            editor_mode: None,
            plugins: None,
            notebooks: None,
            attachments_mb: None,
            sync_errors: None,
            channel: None,
            feature: None,
            source: None,
            count: None,
            duration: None,
        }
    }
}

impl Serialize for Batch {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let events = self
            .events
            .iter()
            .map(|event| {
                let mut props = WireProps::base(&self.device);
                let name = match &event.kind {
                    EventKind::AppActive(active) => {
                        let x = active.extras();
                        props.os_version = Some(active.os_version());
                        props.sync = Some(active.sync());
                        props.ai = Some(active.ai());
                        props.notes = Some(active.notes());
                        props.locale_lang = x.locale_lang.as_ref();
                        props.theme_mode = x.theme_mode;
                        props.theme = x.theme.as_ref();
                        props.editor_mode = x.editor_mode;
                        props.plugins = x.plugins;
                        props.notebooks = x.notebooks;
                        props.attachments_mb = x.attachments_mb;
                        props.sync_errors =
                            Some(SyncErrors::from_count(u64::from(active.sync_errors)));
                        props.channel = x.channel;
                        "app_active"
                    }
                    EventKind::AppOpened => "app_opened",
                    EventKind::Session(length) => {
                        props.duration = Some(*length);
                        "session"
                    }
                    EventKind::FeatureUsed(feature, count) => {
                        props.feature = Some(feature.name());
                        props.source = feature.source();
                        props.count = Some(CountRange::from_count(u64::from(*count)));
                        "feature_used"
                    }
                };
                WireEvent {
                    event: name,
                    day: event.day,
                    props,
                }
            })
            .collect();
        Wire {
            v: 1,
            id: self.id.as_str(),
            events,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::SyncMode;

    /// The page's own example, byte for byte apart from spacing.
    #[test]
    fn writes_the_spec_example() {
        let day = Day::parse("2026-10-30").unwrap();
        let batch = Batch {
            id: Id::parse("a_3f9c0d4e5b6a79881726354433221100").unwrap(),
            device: Device::new(Platform::Macos, Form::Desktop, "0.9.0"),
            events: vec![
                Event {
                    day,
                    kind: EventKind::AppActive(Active::new("15.1", SyncMode::Kanso, false, 500)),
                },
                Event {
                    day,
                    kind: EventKind::FeatureUsed(Feature::Import(ImportSource::Obsidian), 1),
                },
            ],
        };
        let expected = serde_json::json!({
            "v": 1,
            "id": "a_3f9c0d4e5b6a79881726354433221100",
            "events": [
                {
                    "event": "app_active",
                    "day": "2026-10-30",
                    "props": {
                        "platform": "macos",
                        "form": "desktop",
                        "app_version": "0.9.0",
                        "os_version": "15.1",
                        "sync": "kanso",
                        "ai": false,
                        "notes": "100-999",
                        "sync_errors": "0"
                    }
                },
                {
                    "event": "feature_used",
                    "day": "2026-10-30",
                    "props": { "platform": "macos", "form": "desktop", "app_version": "0.9.0", "feature": "import", "source": "obsidian", "count": "1" }
                }
            ]
        });
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&batch.to_json()).unwrap(),
            expected
        );
        // The props keep the page's order.
        let text = String::from_utf8(batch.to_json()).unwrap();
        assert!(text.contains(r#""props":{"platform":"macos","form":"desktop","app_version":"0.9.0","os_version":"15.1","sync":"kanso","ai":false,"notes":"100-999","sync_errors":"0"}"#));
        // A feature without a source has no `source` at all.
        let lock = Batch {
            events: vec![Event {
                day,
                kind: EventKind::FeatureUsed(Feature::Lock, 3),
            }],
            ..batch
        };
        assert!(
            !String::from_utf8(lock.to_json())
                .unwrap()
                .contains("source")
        );
    }
}
