//! Meaning, from the embeddings related notes keep (`embed.rs`): searching
//! notes by what they are about.
//!
//! A search needs its words turned
//! into a vector by the same model the notes were embedded with: the app
//! lends the store its model while it is loaded ([`Store::lend_embedder`]),
//! and a store opened without the app answers [`MeaningSearch::NoModel`].

pub mod model;
pub mod rank;

use std::collections::HashSet;
use std::sync::Arc;

use kanso_core::Note;
use kanso_core::excerpt::{HEAD_CHARS, excerpt_around};

use crate::embed::{embedded_notes, nearest};
use crate::{Embedder, NoteSummary, Result, Store};
use model::MeaningSearch;

impl Store {
    /// Lends the store the app's embedding model while it is loaded
    /// (`None` once it isn't), so searches by meaning can turn their words
    /// into a vector, in the app and through its local API.
    pub fn lend_embedder(&self, embedder: Option<Arc<dyn Embedder>>) {
        *self.embedder.write().expect("embedder poisoned") = embedder;
    }

    /// The lent model, when it is the one the notes are embedded with.
    fn query_embedder(&self, model: &str) -> Option<Arc<dyn Embedder>> {
        self.embedder
            .read()
            .expect("embedder poisoned")
            .clone()
            .filter(|embedder| embedder.model() == model)
    }

    /// The `k` live notes outside the trash nearest in meaning to `query`.
    pub fn search_by_meaning(&self, query: &str, k: usize) -> Result<MeaningSearch> {
        let Some(model) = self.embedding_model()? else {
            return Ok(MeaningSearch::Off);
        };
        let Some(embedder) = self.query_embedder(&model) else {
            return Ok(MeaningSearch::NoModel);
        };
        let words = query.trim();
        if words.is_empty() {
            return Ok(MeaningSearch::Found(Vec::new()));
        }
        let Some(vector) = embedder.embed(&[words.to_owned()]).pop() else {
            return Ok(MeaningSearch::Found(Vec::new()));
        };
        self.read(|conn| {
            let candidates = embedded_notes(conn, &model, &HashSet::new(), false)?;
            Ok(MeaningSearch::Found(nearest(&vector, &candidates, k)))
        })
    }

    /// The live notes with `ids` as the note list shows them, in that
    /// order; any gone meanwhile are left out.
    pub fn summaries_of(&self, ids: &[&str]) -> Result<Vec<NoteSummary>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(note) = self.note(id)? {
                out.push(summary(&note));
            }
        }
        Ok(out)
    }
}

/// A note as a list row, its snippet from the start of its body.
fn summary(note: &Note) -> NoteSummary {
    let locked = note.is_locked();
    let head: String = if locked {
        String::new()
    } else {
        note.body.chars().take(HEAD_CHARS).collect()
    };
    NoteSummary {
        id: note.id.clone(),
        rev: note.rev.clone().unwrap_or_default(),
        title: note.title.clone(),
        snippet: excerpt_around(&note.title, &head, None, &[] as &[&str]),
        book_id: note.book_id.clone(),
        status: note.status,
        tags: note.tags.clone(),
        pinned: note.pinned,
        updated_at: note.updated_at,
        created_at: note.created_at,
        tasks_done: note.num_of_checked_tasks,
        tasks_total: note.num_of_tasks,
        locked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Origin, Put};
    use kanso_core::DocType;
    use serde_json::json;

    /// Counts letters: texts with the same letters point the same way.
    struct Letters;

    impl Embedder for Letters {
        fn model(&self) -> &str {
            "letters"
        }
        fn dim(&self) -> usize {
            26
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts
                .iter()
                .map(|t| {
                    let mut v = vec![0f32; 26];
                    for c in t.to_ascii_lowercase().bytes() {
                        if c.is_ascii_lowercase() {
                            v[(c - b'a') as usize] += 1.;
                        }
                    }
                    v
                })
                .collect()
        }
    }

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:b", "name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        (dir, store)
    }

    fn note(store: &Store, title: &str, body: &str, tags: &[&str]) -> String {
        let tags: Vec<String> = tags
            .iter()
            .map(|t| store.tag_named(t, Origin::Ui).unwrap())
            .collect();
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": "book:b", "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// Embeds whatever is waiting, as the app's indexer does.
    fn index(store: &Store) {
        let queue = store.embedding_queue("letters").unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let rows: Vec<_> = queue
            .pending
            .into_iter()
            .zip(Letters.embed(&texts))
            .map(|(p, v)| (p.id, p.hash, v))
            .collect();
        store.put_embeddings("letters", &rows).unwrap();
    }

    #[test]
    fn a_search_by_meaning_needs_the_feature_and_the_model() {
        let (_dir, store) = open();
        let a = note(&store, "zzz", "zzzz yyy", &[]);
        let b = note(&store, "abc", "aabbcc", &[]);
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::Off
        );
        store.set_embedding_model(Some("letters")).unwrap();
        index(&store);
        // On, but this process has no model.
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::NoModel
        );
        store.lend_embedder(Some(Arc::new(Letters)));
        let MeaningSearch::Found(found) = store.search_by_meaning("zzy", 5).unwrap() else {
            panic!("no search");
        };
        assert_eq!(found[0].id, a);
        assert_eq!(found[1].id, b);
        assert!(found[0].score > found[1].score);
        let rows = store.summaries_of(&[&b, "note:gone", &a]).unwrap();
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["abc", "zzz"]);
        assert_eq!(rows[0].snippet, "aabbcc");
        // A model lent for another embedding is not used.
        store.set_embedding_model(Some("other")).unwrap();
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::NoModel
        );
        store.lend_embedder(None);
    }
}
