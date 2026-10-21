//! What publishing a notebook puts on the web, apart from the sheet
//! (`publish_site.rs`): the notebook's notes and those of the notebooks
//! inside it, locked notes left out, each a page of a small site whose
//! links between published notes work (`kanso_sync::site`).

use std::collections::{HashMap, HashSet};

use kanso_core::links::normalize_title;
use kanso_core::{Book, Clause, Field, Query};
use kanso_render::Palette;
use kanso_render::html::{IndexEntry, NoteRef, Site, attachment_id};
use kanso_store::{SearchParams, Sort, Store, StoreError};
use kanso_sync::Session;
use kanso_sync::share::{self, KansoPublish, Page, ShareApi, ShareError};
use kanso_sync::site::{SiteBundle, SiteError, SiteFile, fingerprint, slug_for};

use crate::library::{book_path, book_tree};

/// A note the site will show.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlannedNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub snippet: String,
    pub slug: String,
    /// The notebook inside the published one it is in, as "Plans › 2026";
    /// `None` for the published notebook's own notes.
    pub group: Option<String>,
}

/// An attachment a published note uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedFile {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

impl PlannedFile {
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }
}

/// What publishing a notebook would put on the web, before anything is
/// rendered: for the sheet to show, and to build from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SitePlan {
    pub book_id: String,
    pub name: String,
    pub notes: Vec<PlannedNote>,
    /// The titles of the locked notes left out.
    pub locked: Vec<String>,
    pub files: Vec<PlannedFile>,
    /// The notebook's link, when it is published.
    pub publish: Option<KansoPublish>,
}

impl SitePlan {
    /// What the site is made from, for telling whether it changed since it
    /// was published (`KansoPublish::is_outdated`, with the name as title).
    pub fn fingerprint(&self) -> String {
        fingerprint(
            self.notes
                .iter()
                .map(|n| (n.id.as_str(), n.title.as_str(), n.body.as_str())),
        )
    }

    /// Whether `publish` shows something other than what the notebook
    /// holds now.
    pub fn is_outdated(&self, publish: &KansoPublish) -> bool {
        publish.is_outdated(&self.name, &self.fingerprint())
    }
}

/// The notebooks at and under `book_id`, in the sidebar's order, each with
/// its path below the published one ("" for itself). Pure.
pub fn notebooks_under(book_id: &str, books: &[Book]) -> Vec<(String, String)> {
    book_tree(books)
        .into_iter()
        .filter_map(|(_, book)| {
            let path = book_path(&book.id, books);
            let at = path.iter().position(|b| b.id == book_id)?;
            let below: Vec<&str> = path[at + 1..].iter().map(|b| b.name.as_str()).collect();
            Some((book.id.clone(), below.join(" › ")))
        })
        .collect()
}

/// The attachment IDs a body links to or shows, in order, once each. Pure.
pub fn attachments_in(body: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    body.match_indices(kanso_render::html::FILE_LINK_PREFIX)
        .filter_map(|(at, _)| attachment_id(&body[at..]))
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

/// Reads what publishing notebook `book_id` would show.
pub fn plan(store: &Store, book_id: &str) -> Result<SitePlan, StoreError> {
    let books = store.books()?;
    let book = books
        .iter()
        .find(|b| b.id == book_id)
        .ok_or(StoreError::NotFound)?;
    let mut plan = SitePlan {
        book_id: book_id.to_owned(),
        name: book.name.clone(),
        publish: book_publish(book),
        ..SitePlan::default()
    };
    let mut slugs: HashSet<String> = HashSet::new();
    let mut files: HashSet<String> = HashSet::new();
    for (book, group) in notebooks_under(book_id, &books) {
        let params = SearchParams {
            query: Query {
                clauses: vec![Clause::field(Field::BookId, &book)],
            },
            sort: Sort::Title,
            descending: false,
            limit: None,
            skip: 0,
            ocr: false,
        };
        for summary in store.search_summaries(&params)? {
            if summary.book_id != book {
                continue;
            }
            if summary.locked {
                plan.locked.push(summary.title.clone());
                continue;
            }
            let Some(note) = store.note(&summary.id)? else {
                continue;
            };
            // Read again: a note locked since the list was read, or whose
            // notebook was, stays out.
            if note.is_locked() || store.note_is_locked(&note.id)? {
                plan.locked.push(note.title.clone());
                continue;
            }
            let slug = slug_for(&note.title, &|s| slugs.contains(s));
            slugs.insert(slug.clone());
            for id in attachments_in(&note.body) {
                if files.insert(id.clone())
                    && let Some(meta) = store.file_meta(&id)?
                {
                    plan.files.push(PlannedFile {
                        id,
                        name: meta.name,
                        content_type: meta.content_type,
                        size: meta.content_length,
                    });
                }
            }
            plan.notes.push(PlannedNote {
                id: note.id,
                title: note.title,
                body: note.body,
                snippet: summary.snippet,
                slug,
                group: (!group.is_empty()).then(|| group.clone()),
            });
        }
    }
    Ok(plan)
}

/// An attachment's type and bytes by ID, when the store has it.
pub type Attachments<'a> = dyn Fn(&str) -> Option<(String, Vec<u8>)> + 'a;

/// The site for `plan`: the index and every note's page in `palette`,
/// links between published notes kept, images in once, and the other
/// attachments a note links to offered on its page. Blocks while math and
/// diagrams are drawn.
pub fn build(
    plan: &SitePlan,
    attachments: &Attachments,
    palette: &Palette,
) -> Result<SiteBundle, SiteError> {
    let mut by_id: HashMap<&str, &str> = HashMap::new();
    let mut by_title: HashMap<String, &str> = HashMap::new();
    for note in &plan.notes {
        by_id.insert(&note.id, &note.slug);
        by_title
            .entry(normalize_title(&note.title))
            .or_insert(&note.slug);
    }
    let resolve = |target: NoteRef<'_>| -> Option<String> {
        match target {
            NoteRef::Id(id) => by_id.get(id).map(|s| (*s).to_owned()),
            NoteRef::Title(title) => by_title
                .get(&normalize_title(title))
                .map(|s| (*s).to_owned()),
        }
    };
    let site = Site {
        resolve: &resolve,
        home: &plan.name,
    };
    // Only the attachments of published notes: a locked note's never.
    let known: HashMap<&str, &PlannedFile> =
        plan.files.iter().map(|f| (f.id.as_str(), f)).collect();
    let attached = |id: &str| known.contains_key(id).then(|| attachments(id)).flatten();
    let mut bundle = SiteBundle::new(&plan.name);
    let mut downloads: Vec<(String, Vec<String>)> = Vec::new();
    for note in &plan.notes {
        let html =
            kanso_render::html::page_in(&note.title, &note.body, palette, &attached, Some(&site));
        bundle.pages.insert(
            note.slug.clone(),
            kanso_sync::site::SitePage {
                title: note.title.clone(),
                html,
                downloads: Vec::new(),
            },
        );
        let others: Vec<String> = attachments_in(&note.body)
            .into_iter()
            .filter(|id| known.get(id.as_str()).is_some_and(|f| !f.is_image()))
            .collect();
        downloads.push((note.slug.clone(), others));
    }
    let entries: Vec<IndexEntry> = plan
        .notes
        .iter()
        .map(|note| IndexEntry {
            slug: note.slug.clone(),
            title: note.title.clone(),
            snippet: note.snippet.clone(),
            group: note.group.clone(),
        })
        .collect();
    bundle.index = kanso_render::html::site_index(&plan.name, &entries, palette);
    // Images: each in the bundle once, the pages pointing at it.
    let images: Vec<SiteFile> = plan
        .files
        .iter()
        .filter(|f| f.is_image())
        .filter_map(|f| {
            let (content_type, bytes) = attached(&f.id)?;
            Some(SiteFile::new(&f.name, &content_type, &bytes))
        })
        .collect();
    bundle.share_images(images);
    // Other attachments: offered on the pages that link to them.
    let mut placed: HashMap<String, usize> = HashMap::new();
    for (slug, ids) in downloads {
        for id in ids {
            let at = match placed.get(&id) {
                Some(&at) => at,
                None => {
                    let Some((content_type, bytes)) = attached(&id) else {
                        continue;
                    };
                    let name = known.get(id.as_str()).map_or("file", |f| f.name.as_str());
                    bundle
                        .files
                        .push(SiteFile::new(name, &content_type, &bytes));
                    placed.insert(id.clone(), bundle.files.len() - 1);
                    bundle.files.len() - 1
                }
            };
            if let Some(page) = bundle.pages.get_mut(&slug) {
                page.downloads.push(at);
            }
        }
    }
    bundle.check()?;
    Ok(bundle)
}

/// Why a notebook couldn't be published.
#[derive(Debug)]
pub enum PublishError {
    /// The site came out inconsistent: a bug, never the notebook's fault.
    Site(SiteError),
    Share(ShareError),
}

impl From<SiteError> for PublishError {
    fn from(err: SiteError) -> PublishError {
        PublishError::Site(err)
    }
}

impl From<ShareError> for PublishError {
    fn from(err: ShareError) -> PublishError {
        PublishError::Share(err)
    }
}

/// Builds the site for `plan` and seals it at the notebook's link: the
/// same ID and key with new ciphertext when it has one that still works,
/// else a new link, as `share::publish` does for a note. The link's content
/// hash is of the notebook's name and `plan.fingerprint()`, so
/// [`SitePlan::is_outdated`] tells when the notebook has changed since.
/// Returns what the notebook should now keep. Blocks.
pub fn publish(
    api: &dyn ShareApi,
    session: &Session,
    plan: &SitePlan,
    store: &Store,
    palette: &Palette,
    expiry: Option<i64>,
    now_ms: i64,
) -> Result<KansoPublish, PublishError> {
    let attachments = |id: &str| {
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    let bundle = build(plan, &attachments, palette)?;
    let bytes = bundle.to_bytes();
    let fingerprint = plan.fingerprint();
    let page = Page {
        html: &bytes,
        title: &plan.name,
        body: &fingerprint,
        rev: "",
    };
    Ok(share::publish(
        api,
        session,
        plan.publish.as_ref(),
        &page,
        expiry,
        now_ms,
    )?)
}

/// What a notebook says it published, when usable.
pub fn book_publish(book: &Book) -> Option<KansoPublish> {
    KansoPublish::from_extra(&book.extra)
}

/// Keeps `publish` in notebook `book_id`, or (with `None`) takes it out.
pub fn store_book_publish(
    store: &Store,
    book_id: &str,
    publish: Option<&KansoPublish>,
) -> anyhow::Result<()> {
    let Some(mut book) = store.books()?.into_iter().find(|b| b.id == book_id) else {
        return Ok(());
    };
    KansoPublish::store(publish, &mut book.extra);
    store.put(
        kanso_core::DocType::Book,
        serde_json::to_value(&book)?,
        kanso_store::Put::Upsert,
        kanso_store::Origin::Ui,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{attachments_in, book_publish, build, notebooks_under, plan, store_book_publish};
    use kanso_core::DocType;
    use kanso_core::links::{FILE_LINK_PREFIX as FILE, NOTE_LINK_PREFIX as NOTE};
    use kanso_render::Palette;
    use kanso_store::{Origin, Put, Store};
    use kanso_sync::share::{KansoPublish, ShareKey, new_share_id, open_share, seal_share};
    use kanso_sync::site::{SiteBundle, file_sources, page_links};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        for (id, name, parent) in [
            ("book:work", "Work", None),
            ("book:old", "Old", Some("book:work")),
            ("book:home", "Home", None),
        ] {
            store
                .put(
                    DocType::Book,
                    json!({"_id": id, "name": name, "parentBookId": parent}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        store
            .put(
                DocType::File,
                json!({"_id": "file:png", "name": "dot.png", "contentType": "image/png",
                    "_attachments": {"index": {"content_type": "image/png", "data": "iVBORw=="}}}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        (dir, store)
    }

    fn note(store: &Store, id: &str, title: &str, body: &str, book: &str) {
        store
            .put(
                DocType::Note,
                json!({"_id": id, "title": title, "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }

    /// Locks note `id` as the app does: its text only in the sealed field.
    fn lock(store: &Store, id: &str) {
        let mut note = store.note(id).unwrap().unwrap();
        note.body = String::new();
        note.extra.insert(
            kanso_core::model::LOCK_FIELD.into(),
            json!({"v": 1, "sealed": "c2VjcmV0"}),
        );
        store
            .lock_note(serde_json::to_value(&note).unwrap())
            .unwrap();
    }

    fn attachments(store: &Store) -> impl Fn(&str) -> Option<(String, Vec<u8>)> + '_ {
        |id| {
            store
                .attachment(id)
                .ok()
                .flatten()
                .map(|(t, a)| (t, a.data))
        }
    }

    #[test]
    fn a_notebook_becomes_a_site() {
        let (_dir, store) = store();
        note(
            &store,
            "note:a",
            "Plan",
            &format!("See [[Ideas]] and [[Diary]] and [[Elsewhere]].\n\n![dot]({FILE}png)\n"),
            "book:work",
        );
        note(
            &store,
            "note:b",
            "Ideas",
            &format!("Back to [plan]({NOTE}a).\n\n![dot]({FILE}png)\n"),
            "book:old",
        );
        note(&store, "note:c", "Diary", "The secret diary.", "book:work");
        note(
            &store,
            "note:d",
            "Elsewhere",
            "Not in this notebook.",
            "book:home",
        );
        lock(&store, "note:c");
        let plan = plan(&store, "book:work").unwrap();
        let titles: Vec<&str> = plan.notes.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, ["Plan", "Ideas"]);
        assert_eq!(plan.locked, ["Diary"]);
        assert_eq!(plan.notes[1].group.as_deref(), Some("Old"));
        assert_eq!(plan.files.len(), 1);

        let bundle = build(&plan, &attachments(&store), &Palette::LIGHT).unwrap();
        assert_eq!(bundle.title, "Work");
        let plan_page = &bundle.pages["plan"].html;
        assert!(plan_page.contains(r#"href="kanso-page:ideas""#));
        // Locked and other notebooks' notes are text, not links.
        assert!(plan_page.contains(r#"<span class="kanso-wikilink">Diary</span>"#));
        assert!(plan_page.contains(r#"<span class="kanso-wikilink">Elsewhere</span>"#));
        assert!(
            bundle.pages["ideas"]
                .html
                .contains(r#"<a href="kanso-page:plan">plan</a>"#)
        );
        // The image is in once, for both pages.
        assert_eq!(bundle.files.len(), 1);
        assert_eq!(file_sources(plan_page), [0]);
        assert_eq!(file_sources(&bundle.pages["ideas"].html), [0]);
        let all = String::from_utf8(bundle.to_bytes()).unwrap();
        assert!(!all.contains("secret diary"));
        assert!(bundle.index.contains("<h2>Old</h2>"));

        // The fingerprint follows edits.
        let before = plan.fingerprint();
        store
            .put(
                DocType::Note,
                json!({"_id": "note:b", "body": "Changed."}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_ne!(
            super::plan(&store, "book:work").unwrap().fingerprint(),
            before
        );
    }

    /// An attachment that isn't an image (from an import) is offered on
    /// the pages that link to it, and is in the site once.
    #[test]
    fn other_attachments_are_offered_on_their_pages() {
        let planned = |id: &str, title: &str, body: &str| super::PlannedNote {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            snippet: String::new(),
            slug: title.to_lowercase(),
            group: None,
        };
        let plan = super::SitePlan {
            book_id: "book:x".into(),
            name: "X".into(),
            notes: vec![
                planned("note:a", "A", &format!("[the pdf]({FILE}pdf)")),
                planned(
                    "note:b",
                    "B",
                    &format!("[again]({FILE}pdf) [gone]({FILE}zip)"),
                ),
            ],
            locked: Vec::new(),
            publish: None,
            files: vec![super::PlannedFile {
                id: "file:pdf".into(),
                name: "plan.pdf".into(),
                content_type: "application/pdf".into(),
                size: 3,
            }],
        };
        let read = |id: &str| {
            (id == "file:pdf" || id == "file:zip")
                .then(|| ("application/pdf".to_owned(), b"pdf".to_vec()))
        };
        let bundle = build(&plan, &read, &Palette::LIGHT).unwrap();
        assert_eq!(
            bundle.files.len(),
            1,
            "a file the plan doesn't name stays out"
        );
        assert_eq!(bundle.files[0].name, "plan.pdf");
        assert_eq!(bundle.files[0].bytes().unwrap(), b"pdf");
        assert_eq!(bundle.pages["a"].downloads, [0]);
        assert_eq!(bundle.pages["b"].downloads, [0]);
    }

    #[test]
    fn a_notebook_keeps_its_link_beside_its_other_fields() {
        let (_dir, store) = store();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:work", "kansoDailyTemplate": "note:t"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let publish = KansoPublish {
            share_id: new_share_id(),
            key: ShareKey::generate().to_text(),
            published_at: 5,
            published_rev: String::new(),
            expires_at: None,
            content_hash: None,
            short_url: None,
            other: Default::default(),
        };
        store_book_publish(&store, "book:work", Some(&publish)).unwrap();
        let book = store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.id == "book:work")
            .unwrap();
        assert_eq!(book_publish(&book), Some(publish));
        assert_eq!(book.extra["kansoDailyTemplate"], "note:t");
        assert_eq!(book.parent_book_id, None);
        store_book_publish(&store, "book:work", None).unwrap();
        let book = store
            .books()
            .unwrap()
            .into_iter()
            .find(|b| b.id == "book:work")
            .unwrap();
        assert_eq!(book_publish(&book), None);
        assert!(!book.extra.contains_key("kansoPublish"));
        assert_eq!(book.extra["kansoDailyTemplate"], "note:t");
    }

    #[test]
    fn notebooks_and_attachments() {
        let (_dir, store) = store();
        let books = store.books().unwrap();
        let under: Vec<(String, String)> = notebooks_under("book:work", &books);
        assert_eq!(
            under,
            [
                ("book:work".to_owned(), String::new()),
                ("book:old".to_owned(), "Old".to_owned())
            ]
        );
        assert_eq!(
            attachments_in(&format!("![a]({FILE}x) [b]({FILE}y) ![c]({FILE}x)")),
            ["file:x", "file:y"]
        );
    }

    proptest! {
        #![proptest_config(config())]

        /// Any notebook of notes linking to each other, some locked: the
        /// site seals and opens to the same bundle, every link in it
        /// reaches one of its pages, and no locked note's title as a page
        /// or text is anywhere in it.
        #[test]
        fn prop_sites_hold_only_unlocked_notes_and_their_links_resolve(
            notes in prop::collection::vec(("[A-Za-z]{1,8}", any::<bool>(), prop::collection::vec(0usize..8, 0..4)), 1..7),
        ) {
            let (_dir, store) = store();
            for (ix, (title, _, links)) in notes.iter().enumerate() {
                let body: String = links
                    .iter()
                    .map(|l| match notes.get(*l) {
                        Some((t, _, _)) if l % 2 == 0 => format!("[[{t}]] "),
                        Some(_) => format!("[x]({NOTE}n{l}) "),
                        None => "[[Nowhere]] ".to_owned(),
                    })
                    .collect::<String>() + &format!("secret{ix}");
                note(&store, &format!("note:n{ix}"), title, &body, "book:work");
            }
            for (ix, (_, locked, _)) in notes.iter().enumerate() {
                if *locked {
                    lock(&store, &format!("note:n{ix}"));
                }
            }
            let plan = plan(&store, "book:work").unwrap();
            prop_assert_eq!(plan.notes.len(), notes.iter().filter(|n| !n.1).count());
            let bundle = build(&plan, &attachments(&store), &Palette::LIGHT).unwrap();
            prop_assert!(bundle.check().is_ok());
            let key = ShareKey::generate();
            let id = new_share_id();
            let opened = open_share(&key, &id, &seal_share(&key, &id, &bundle.to_bytes()).unwrap()).unwrap();
            let opened = SiteBundle::from_bytes(&opened).unwrap();
            prop_assert_eq!(&opened, &bundle);
            for html in std::iter::once(&opened.index).chain(opened.pages.values().map(|p| &p.html)) {
                for slug in page_links(html) {
                    prop_assert!(slug.is_empty() || opened.pages.contains_key(&slug));
                }
            }
            let text = String::from_utf8(opened.to_bytes()).unwrap();
            for (ix, (_, locked, _)) in notes.iter().enumerate() {
                // Ends with a quote or a tag: "secret1" isn't read in "secret10".
                let found = text.contains(&format!("secret{ix}<")) || text.contains(&format!("secret{ix}\""));
                prop_assert_eq!(found, !locked, "note {}", ix);
            }
        }
    }
}
