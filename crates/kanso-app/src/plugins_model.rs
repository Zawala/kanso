//! The shapes the plugin views draw from: a command in the palette, an
//! install waiting on its sheet, a batch waiting on review, a panel and
//! its inputs, and what the directory has.

use std::collections::BTreeMap;

use kanso_plugins::batch::{Batch, ReviewNote};
use kanso_plugins::consent::Signed;
use kanso_plugins::model::{Grant, Installed, Source};
use kanso_plugins::panel_model::Panel;
use kanso_plugins::protocol::CommandSpec;
use kanso_plugins::registry_model::{Bundle, Entry, Index, Offer};
use serde_json::Value;

/// A plugin's command, as the palette lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCommand {
    pub plugin_id: String,
    pub plugin_name: String,
    pub spec: CommandSpec,
}

/// A bundle checked and waiting for the person on the install sheet.
#[derive(Clone, Debug)]
pub struct InstallPlan {
    pub bundle: Bundle,
    pub source: Source,
    pub signed: Signed,
    /// Installed (or well-known) plugin IDs within two edits of this one.
    pub lookalikes: Vec<String>,
    /// The plugin this replaces, when it is already installed.
    pub replacing: Option<Installed>,
    /// What the sheet starts from.
    pub grant: Grant,
    /// The update this is, when it is one: the permission diff and whether
    /// the author's key changed.
    pub offer: Option<Offer>,
}

impl InstallPlan {
    pub fn id(&self) -> &str {
        &self.bundle.manifest.id
    }

    /// Whether the install button is the red "Install anyway (key changed)".
    pub fn key_changed(&self) -> bool {
        self.offer.as_ref().is_some_and(|o| o.key_changed)
            || self.replacing.as_ref().is_some_and(|r| {
                !r.is_developer() && r.pinned_key != self.bundle.manifest.publisher.key
            })
    }
}

/// A command's changes, waiting for Apply on the review sheet.
#[derive(Clone, Debug)]
pub struct PendingReview {
    pub plugin_id: String,
    pub plugin_name: String,
    pub batch: Batch,
    pub notes: Vec<ReviewNote>,
}

impl PendingReview {
    /// "Word count wants to change 37 notes".
    pub fn heading(&self) -> String {
        let changed = self.batch.edits.len();
        let made = self.batch.creates.len();
        let notes = |n: usize| if n == 1 { "note" } else { "notes" };
        match (changed, made) {
            (0, made) => format!("{} wants to make {made} {}", self.plugin_name, notes(made)),
            (changed, 0) => format!(
                "{} wants to change {changed} {}",
                self.plugin_name,
                notes(changed)
            ),
            (changed, made) => format!(
                "{} wants to change {changed} {} and make {made}",
                self.plugin_name,
                notes(changed)
            ),
        }
    }
}

/// A plugin's panel in the inspector, and what its inputs hold.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelState {
    pub panel: Panel,
    pub developer: bool,
    /// Input, toggle and select values by ID, as the person left them.
    pub values: BTreeMap<String, Value>,
    /// A press is running.
    pub busy: bool,
}

/// What Settings › Plugins › Browse shows.
#[derive(Clone, Debug, Default)]
pub enum Directory {
    /// This build has no directory key.
    #[default]
    NotSetUp,
    /// Not fetched yet this session.
    Idle,
    Loading,
    Loaded(Index),
    Failed(String),
}

impl Directory {
    pub fn index(&self) -> Option<&Index> {
        match self {
            Directory::Loaded(index) => Some(index),
            _ => None,
        }
    }

    /// The latest version of each plugin matching `query`.
    pub fn search(&self, query: &str) -> Vec<Entry> {
        self.index()
            .map(|index| {
                kanso_plugins::registry::search(index, query)
                    .into_iter()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// How a plugin stands, in Settings' words. Pure.
pub fn status_line(installed: &Installed, running: bool, safe_mode: bool) -> String {
    use kanso_plugins::model::Disabled;
    match &installed.disabled {
        Some(Disabled::ByYou) => "Off".to_owned(),
        Some(Disabled::Crashed) => "Turned off: it crashed twice in ten minutes".to_owned(),
        Some(Disabled::FailedToStart) => "Turned off: it failed to start three times".to_owned(),
        Some(Disabled::Refused) => {
            "Turned off: it asked three times for something it isn't allowed".to_owned()
        }
        Some(Disabled::Revoked { reason }) => format!("Withdrawn by the directory: {reason}"),
        Some(Disabled::Blocked) => "Blocked on this Mac".to_owned(),
        Some(Disabled::NeedsNewerKanso { version }) => format!("Needs Kanso {version} or later"),
        None if safe_mode => "Off for this session (safe mode)".to_owned(),
        None if running => "Running".to_owned(),
        None => "On".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn review(changed: usize, made: usize) -> PendingReview {
        let mut batch = Batch::new();
        for i in 0..changed {
            batch
                .stage_edit(kanso_store::PluginEdit {
                    id: format!("note:{i}"),
                    base_rev: "1-a".into(),
                    ..Default::default()
                })
                .unwrap();
        }
        for _ in 0..made {
            batch
                .stage_create(kanso_store::PluginCreate {
                    book_id: "book:a".into(),
                    title: String::new(),
                    body: String::new(),
                })
                .unwrap();
        }
        PendingReview {
            plugin_id: "com.example.wordcount".into(),
            plugin_name: "Word count".into(),
            batch,
            notes: Vec::new(),
        }
    }

    #[test]
    fn the_review_heading_counts() {
        assert_eq!(
            review(37, 0).heading(),
            "Word count wants to change 37 notes"
        );
        assert_eq!(review(1, 0).heading(), "Word count wants to change 1 note");
        assert_eq!(review(0, 2).heading(), "Word count wants to make 2 notes");
        assert_eq!(
            review(2, 1).heading(),
            "Word count wants to change 2 notes and make 1"
        );
    }

    proptest! {
        /// Invariant: the heading always names the plugin and the number of
        /// notes changed.
        #[test]
        fn prop_the_heading_names_the_counts(changed in 0usize..40, made in 0usize..5) {
            prop_assume!(changed + made > 0);
            let heading = review(changed, made).heading();
            prop_assert!(heading.starts_with("Word count wants to "));
            if changed > 0 {
                let counted = format!("change {changed} ");
                prop_assert!(heading.contains(&counted));
            }
        }
    }
}
