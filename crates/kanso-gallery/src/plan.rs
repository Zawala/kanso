//! What an automatic check does with each installed gallery theme.
//!
//! The plan is pure: it reads the verified catalog, the installed records and
//! the bytes actually on disk, and returns the actions to take. Nothing is
//! downloaded here, so the rule that a no index which fails the signature or
//! serial is ever applied lives in `check_index`, and the rule that an edited
//! copy is never overwritten lives here.

use semver::Version;

use crate::index::{Index, Kind, Theme};

/// A theme Kanso installed, as `state.json` records it. `digest` is what
/// Kanso wrote (a color file's SHA-256, an icon folder's tree digest);
/// `current_digest` is what is on disk now, or `None` when it is missing.
/// Reconciling the record against the disk is what makes an edited copy
/// visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub id: String,
    pub kind: Kind,
    pub version: u64,
    pub name: String,
    pub family: Option<String>,
    pub digest: String,
    pub current_digest: Option<String>,
    pub installed_at: String,
}

impl Installed {
    /// Whether the installed bytes are still the ones Kanso wrote. Pure.
    pub fn is_unedited(&self) -> bool {
        self.current_digest.as_deref() == Some(self.digest.as_str())
    }

    /// Whether the installed files are gone. Pure.
    pub fn is_missing(&self) -> bool {
        self.current_digest.is_none()
    }
}

/// What to do with one theme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Fetch the signed version, check it and swap it in. Only ever for an
    /// unedited copy.
    Update(Box<Theme>),
    /// Withdraw a theme the catalog revoked: delete the installed copy and
    /// fall back if it was in use.
    Remove {
        id: String,
        kind: Kind,
        reason: String,
    },
    /// A revoked theme whose copy was edited. Until the owner decides
    /// whether a revocation may delete edits, it is kept on disk, unselected
    /// and not loaded, and the person is told why.
    Disable {
        id: String,
        kind: Kind,
        reason: String,
    },
}

impl Action {
    pub fn id(&self) -> &str {
        match self {
            Action::Update(theme) => &theme.id,
            Action::Remove { id, .. } | Action::Disable { id, .. } => id,
        }
    }
}

/// Why a check left an installed theme alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skipped {
    /// The installed copy and the catalog agree.
    UpToDate,
    /// The copy on disk isn't what Kanso wrote; never overwritten.
    Edited,
    /// The copy on disk is gone; nothing re-installs it automatically.
    Missing,
    /// The catalog no longer lists it; it stays installed, no longer
    /// updated.
    NotInGallery,
    /// The catalog's entry needs a newer Kanso.
    NeedsNewerKanso,
}

/// An action set and the themes it deliberately left alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub actions: Vec<Action>,
    pub skipped: Vec<(String, Skipped)>,
}

impl Plan {
    pub fn skipped(&self, id: &str) -> Option<Skipped> {
        self.skipped
            .iter()
            .find(|(other, _)| other == id)
            .map(|(_, why)| *why)
    }

    /// The action for `id`, if any.
    pub fn action(&self, id: &str) -> Option<&Action> {
        self.actions.iter().find(|action| action.id() == id)
    }
}

/// What the plan decides for an installed record, without taking an action.
/// The Browse cards use it for their badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanState {
    /// Installed and current.
    Installed,
    /// A newer signed version is ready.
    Update,
    /// The copy on disk isn't the one Kanso wrote.
    Edited,
    /// The copy on disk is gone.
    Missing,
    /// No longer in the catalog, but not revoked.
    NotInGallery,
    /// Signed as revoked.
    Revoked,
    /// The catalog's entry needs a newer Kanso.
    NeedsNewerKanso,
}

/// Plans `records` against `index` for this Kanso `current`. Pure: no file
/// or network access.
///
/// A revoked theme is removed when its copy is unedited, and disabled (kept,
/// unselected) when it was edited, until the owner decides that case. A
/// theme with a newer `version` updates only while the copy on disk is the
/// one Kanso wrote; anything else is left alone. Nothing is ever
/// downgraded, and nothing absent from the catalog is updated.
pub fn plan(index: &Index, records: &[Installed], current: &Version) -> Plan {
    let mut out = Plan::default();
    for record in records {
        if let Some(revoked) = index.revoked(&record.id) {
            if record.is_missing() || record.is_unedited() {
                out.actions.push(Action::Remove {
                    id: record.id.clone(),
                    kind: record.kind,
                    reason: revoked.reason.clone(),
                });
            } else {
                out.actions.push(Action::Disable {
                    id: record.id.clone(),
                    kind: record.kind,
                    reason: revoked.reason.clone(),
                });
            }
            continue;
        }
        let Some(theme) = index.theme(&record.id) else {
            out.skipped.push((record.id.clone(), Skipped::NotInGallery));
            continue;
        };
        let state = state_for(record, theme, current);
        match state {
            PlanState::Update => out.actions.push(Action::Update(Box::new(theme.clone()))),
            PlanState::Edited => out.skipped.push((record.id.clone(), Skipped::Edited)),
            PlanState::Missing => out.skipped.push((record.id.clone(), Skipped::Missing)),
            PlanState::NeedsNewerKanso => out
                .skipped
                .push((record.id.clone(), Skipped::NeedsNewerKanso)),
            PlanState::Installed => out.skipped.push((record.id.clone(), Skipped::UpToDate)),
            PlanState::Revoked | PlanState::NotInGallery => {}
        }
    }
    out
}

/// The badge an installed record wears against `index`. Pure.
pub fn state_for(record: &Installed, theme: &Theme, current: &Version) -> PlanState {
    if !theme.runs_on(current) {
        return PlanState::NeedsNewerKanso;
    }
    if record.is_missing() {
        return PlanState::Missing;
    }
    if !record.is_unedited() {
        return PlanState::Edited;
    }
    if theme.version > record.version {
        PlanState::Update
    } else {
        // Never downgrade: an older catalog version is simply current.
        PlanState::Installed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{Appearance, FileRef, Revoked, SCHEMA};
    use proptest::prelude::*;

    fn theme(id: &str, version: u64) -> Theme {
        Theme {
            id: id.into(),
            kind: Kind::Color,
            family: None,
            version,
            name: id.into(),
            appearance: Some(Appearance::Dark),
            author: "Author".into(),
            license: "MIT".into(),
            source: "https://example.com".into(),
            description: "A theme.".into(),
            min_kanso: "0.1.0".into(),
            file: Some(FileRef {
                path: format!("themes/{id}.json"),
                sha256: "ab".repeat(32),
                size: 10,
            }),
            bundle: None,
            preview: Vec::new(),
            colors: None,
        }
    }

    fn index(themes: Vec<Theme>, revoked: Vec<Revoked>) -> Index {
        Index {
            schema: SCHEMA,
            serial: 1,
            generated_at: "2026-10-23T12:00:00Z".into(),
            source_commit: "8d96298839a6176210cd1e3b376b82ffefc70ab3".into(),
            signing_key_id: "x".into(),
            themes,
            revoked,
        }
    }

    fn record(id: &str, version: u64, digest: &str, current: Option<&str>) -> Installed {
        Installed {
            id: id.into(),
            kind: Kind::Color,
            version,
            name: id.into(),
            family: None,
            digest: digest.into(),
            current_digest: current.map(str::to_owned),
            installed_at: "2026-10-23T12:00:00Z".into(),
        }
    }

    fn current() -> Version {
        Version::new(0, 9, 0)
    }

    #[test]
    fn a_newer_unedited_copy_updates() {
        let index = index(vec![theme("nord", 3)], Vec::new());
        let plan = plan(&index, &[record("nord", 2, "aa", Some("aa"))], &current());
        assert!(matches!(plan.actions.as_slice(), [Action::Update(t)] if t.version == 3));
    }

    #[test]
    fn an_edited_copy_is_never_updated() {
        let index = index(vec![theme("nord", 3)], Vec::new());
        let plan = plan(&index, &[record("nord", 2, "aa", Some("bb"))], &current());
        assert!(plan.actions.is_empty());
        assert_eq!(plan.skipped("nord"), Some(Skipped::Edited));
    }

    #[test]
    fn a_missing_copy_is_not_reinstalled_automatically() {
        let index = index(vec![theme("nord", 3)], Vec::new());
        let plan = plan(&index, &[record("nord", 2, "aa", None)], &current());
        assert!(plan.actions.is_empty());
        assert_eq!(plan.skipped("nord"), Some(Skipped::Missing));
    }

    #[test]
    fn a_theme_absent_upstream_is_kept() {
        let index = index(Vec::new(), Vec::new());
        let plan = plan(&index, &[record("nord", 2, "aa", Some("aa"))], &current());
        assert!(plan.actions.is_empty());
        assert_eq!(plan.skipped("nord"), Some(Skipped::NotInGallery));
    }

    #[test]
    fn a_revoked_theme_is_removed_and_an_edited_one_is_only_disabled() {
        let revoked = vec![Revoked {
            id: "nord".into(),
            reason: "Withdrawn.".into(),
            since: "2026-10-01T00:00:00Z".into(),
        }];
        let index = index(Vec::new(), revoked);
        let unedited = plan(&index, &[record("nord", 2, "aa", Some("aa"))], &current());
        assert!(matches!(
            unedited.actions.as_slice(),
            [Action::Remove { .. }]
        ));
        let edited = plan(&index, &[record("nord", 2, "aa", Some("bb"))], &current());
        assert!(matches!(
            edited.actions.as_slice(),
            [Action::Disable { .. }]
        ));
    }

    #[test]
    fn a_newer_min_kanso_is_skipped_without_downgrading() {
        let mut theme = theme("nord", 3);
        theme.min_kanso = "9.0.0".into();
        let index = index(vec![theme], Vec::new());
        let plan = plan(&index, &[record("nord", 2, "aa", Some("aa"))], &current());
        assert!(plan.actions.is_empty());
        assert_eq!(plan.skipped("nord"), Some(Skipped::NeedsNewerKanso));
    }

    proptest! {
        /// Whatever the bytes on disk, an edited copy never produces an
        /// update for that theme.
        #[test]
        fn prop_an_edited_copy_is_never_overwritten(
            recorded in "[0-9a-f]{1,16}",
            on_disk in "[0-9a-f]{1,16}",
            installed_version in 0u64..50,
            catalogue_version in 0u64..50,
        ) {
            prop_assume!(recorded != on_disk);
            let index = index(vec![theme("nord", catalogue_version)], Vec::new());
            let plan = plan(
                &index,
                &[record("nord", installed_version, &recorded, Some(&on_disk))],
                &current(),
            );
            prop_assert!(plan.actions.is_empty(), "{:?}", plan);
            prop_assert_eq!(plan.skipped("nord"), Some(Skipped::Edited));
        }

        /// No catalog ever downgrades an installed theme.
        #[test]
        fn prop_never_downgrades(
            installed_version in 1u64..100,
            catalogue_version in 1u64..100,
        ) {
            let index = index(vec![theme("nord", catalogue_version)], Vec::new());
            let plan = plan(
                &index,
                &[record("nord", installed_version, "aa", Some("aa"))],
                &current(),
            );
            if catalogue_version > installed_version {
                prop_assert!(matches!(plan.actions.as_slice(), [Action::Update(_)]));
            } else {
                prop_assert!(plan.actions.is_empty());
            }
        }

        /// A revocation always wins: an unedited copy is removed, an edited
        /// one disabled, and neither is ever updated.
        #[test]
        fn prop_revocation_always_acts(
            edited in any::<bool>(),
            catalogue_version in 1u64..50,
        ) {
            let index = index(
                Vec::new(),
                vec![Revoked {
                    id: "nord".into(),
                    reason: "Withdrawn.".into(),
                    since: "2026-10-01T00:00:00Z".into(),
                }],
            );
            let on_disk = if edited { "bb" } else { "aa" };
            let plan = plan(
                &index,
                &[record("nord", 0, "aa", Some(on_disk))],
                &current(),
            );
            let _ = catalogue_version;
            prop_assert_eq!(plan.actions.len(), 1);
            match &plan.actions[0] {
                Action::Remove { .. } => prop_assert!(!edited),
                Action::Disable { .. } => prop_assert!(edited),
                Action::Update(_) => prop_assert!(false, "a revocation never updates"),
            }
        }
    }
}
