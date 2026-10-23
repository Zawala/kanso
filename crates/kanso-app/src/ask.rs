//! What a question searches and what it sends: full-text hits and the notes
//! nearest by meaning, merged, with locked notes, the trash and notes tagged
//! `#ai-exclude` left out, and the best passages of what remains.
//!
//! Retrieval is all local. Nothing here calls a model; it only chooses the
//! excerpts a question may send.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

use kanso_ai::{ChatProvider, Event, Excerpt, Prompt};
use kanso_core::query::{Clause, Query};
use kanso_store::{MeaningSearch, SearchParams, Store};

/// How many notes one question may send.
pub const MAX_NOTES: usize = 8;
/// The prompt's character budget.
pub const BUDGET: usize = kanso_ai::DEFAULT_BUDGET;
/// The tag whose notes are never sent, in any case and with or without `#`.
pub const EXCLUDE_TAG: &str = "ai-exclude";

/// How many question words are searched for, and how many hits each gets.
const MAX_TERMS: usize = 8;
const TEXT_RESULTS: usize = 12;
const MEANING_RESULTS: usize = 20;
/// Reciprocal-rank fusion's constant: how quickly lower ranks lose weight.
const RRF_K: f64 = 60.;
/// A passage is at least this long where there is room, and a note gives at
/// most this much unless it is the only one.
const MIN_PER_NOTE: usize = 400;
const MAX_PER_NOTE: usize = 6000;

/// A note as retrieval sees it. The flags are decided by the store; the
/// exclusions are decided here, in [`eligible`] alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub id: String,
    pub title: String,
    pub body: String,
    /// Tag names, as the person writes them.
    pub tags: Vec<String>,
    /// Sealed, or in a locked notebook, even if unlocked right now.
    pub locked: bool,
    pub trashed: bool,
}

/// Whether a note may be sent: not locked, not in the trash, and not tagged
/// `#ai-exclude`. This is the one place the exclusions are decided.
pub fn eligible(candidate: &Candidate) -> bool {
    !candidate.locked && !candidate.trashed && !candidate.tags.iter().any(|tag| is_exclude_tag(tag))
}

/// Whether `tag` is the exclusion tag, however it is written.
pub fn is_exclude_tag(tag: &str) -> bool {
    tag.trim()
        .trim_start_matches('#')
        .eq_ignore_ascii_case(EXCLUDE_TAG)
}

/// The words of a question worth searching for, without the ones every
/// question has. Falls back to the question itself when it is all such words.
pub fn question_terms(question: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "if", "then", "than", "so", "such", "what", "which",
        "who", "whom", "whose", "how", "why", "when", "where", "is", "are", "was", "were", "be",
        "been", "being", "am", "do", "does", "did", "of", "in", "on", "at", "to", "for", "from",
        "with", "about", "into", "over", "under", "again", "further", "your", "you", "my", "me",
        "i", "we", "our", "it", "its", "this", "that", "these", "those", "can", "could", "should",
        "would", "will", "shall", "might", "must", "have", "has", "had", "not", "no", "as", "by",
        "any", "all", "some", "there", "their", "them", "they", "he", "she", "his", "her",
    ];
    let mut seen = HashSet::new();
    let mut terms = Vec::new();
    for word in question.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-') {
        let word = word.trim_matches('\'').to_lowercase();
        if word.chars().count() < 3 || STOP.contains(&word.as_str()) || !seen.insert(word.clone()) {
            continue;
        }
        terms.push(word);
        if terms.len() == MAX_TERMS {
            break;
        }
    }
    if terms.is_empty() {
        let whole = question.trim().to_lowercase();
        if !whole.is_empty() {
            terms.push(whole);
        }
    }
    terms
}

/// Merges ranked ID lists by reciprocal-rank fusion: a note high in several
/// lists comes first. `k` is the most returned.
pub fn fuse(ranked: &[Vec<String>], k: usize) -> Vec<String> {
    let mut scores: HashMap<&str, f64> = HashMap::new();
    for list in ranked {
        for (rank, id) in list.iter().enumerate() {
            *scores.entry(id.as_str()).or_default() += 1. / (RRF_K + rank as f64 + 1.);
        }
    }
    let mut merged: Vec<(&str, f64)> = scores.into_iter().collect();
    merged.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    merged
        .into_iter()
        .take(k)
        .map(|(id, _)| id.to_owned())
        .collect()
}

/// The notes a question may send, best first. The exclusions are applied
/// here before anything is read out.
pub fn select(
    question: &str,
    candidates: &[Candidate],
    limit: usize,
    budget: usize,
) -> Vec<Excerpt> {
    let allowed: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| eligible(candidate))
        .take(limit)
        .collect();
    if allowed.is_empty() || question.trim().is_empty() {
        return Vec::new();
    }
    let terms = question_terms(question);
    let per_note = if allowed.len() == 1 {
        budget
    } else {
        (budget / allowed.len()).clamp(MIN_PER_NOTE, MAX_PER_NOTE)
    };
    let mut used = 0;
    let mut chosen = Vec::new();
    for candidate in allowed {
        let room = per_note.min(budget.saturating_sub(used));
        if room == 0 {
            break;
        }
        let text = best_text(&candidate.body, &terms, room);
        if text.trim().is_empty() {
            continue;
        }
        used += text.chars().count();
        chosen.push(Excerpt {
            id: candidate.id.clone(),
            title: candidate.title.clone(),
            text,
        });
    }
    chosen
}

/// The passages of `body` that best match `terms`, joined, within `room`
/// characters. A note with no matching words still gives its opening.
pub fn best_text(body: &str, terms: &[String], room: usize) -> String {
    if body.trim().is_empty() || room == 0 {
        return String::new();
    }
    let passages = passages(body);
    let mut ranked: Vec<(usize, usize)> = passages
        .iter()
        .enumerate()
        .map(|(ix, passage)| (overlap(passage, terms), ix))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut text = String::new();
    for (score, ix) in ranked {
        if score == 0 && !text.is_empty() {
            break;
        }
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        let left = room.saturating_sub(text.chars().count());
        if left == 0 {
            break;
        }
        let passage = passage_around(&passages[ix], terms, left);
        text.push_str(&passage);
        if text.chars().count() >= room {
            break;
        }
    }
    kanso_ai::prompt::truncate(&text, room)
}

/// How many times any of `terms` appears in `text`, in any case.
fn overlap(text: &str, terms: &[String]) -> usize {
    let lower = text.to_lowercase();
    terms.iter().map(|term| lower.matches(term).count()).sum()
}

/// The paragraphs of a body: runs of lines between blank ones.
fn passages(body: &str) -> Vec<String> {
    let mut passages = Vec::new();
    let mut current = String::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            if !current.trim().is_empty() {
                passages.push(current.trim_end().to_owned());
            }
            current.clear();
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    if !current.trim().is_empty() {
        passages.push(current.trim_end().to_owned());
    }
    if passages.is_empty() && !body.trim().is_empty() {
        passages.push(body.trim().to_owned());
    }
    passages
}

/// `passage`, or the part of it around the first matching term when it is
/// longer than `room`.
fn passage_around(passage: &str, terms: &[String], room: usize) -> String {
    if passage.chars().count() <= room {
        return passage.to_owned();
    }
    let lower = passage.to_lowercase();
    let at = terms
        .iter()
        .filter_map(|term| lower.find(term))
        .min()
        .unwrap_or(0);
    let chars: Vec<char> = passage.chars().collect();
    let at = passage[..at].chars().count();
    let start = at.saturating_sub(room / 4).min(chars.len());
    let end = (start + room).min(chars.len());
    chars[start..end].iter().collect()
}

/// Runs `provider`'s answer to `prompt`, handing each event to `on_event`
/// until the answer ends, fails, or `cancel` is set. The iterator is
/// dropped on cancel, which closes the request.
pub fn stream_answer(
    provider: &dyn ChatProvider,
    prompt: &Prompt,
    cancel: &AtomicBool,
    mut on_event: impl FnMut(Event),
) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    let events = match provider.stream(prompt) {
        Ok(events) => events,
        Err(err) => {
            on_event(Event::Error(err.to_string()));
            return;
        }
    };
    for event in events {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let last = matches!(event, Event::Done | Event::Error(_));
        on_event(event);
        if last {
            return;
        }
    }
}

/// `answer` with each `[[Title]]` that names one of `used` written as a link
/// to that note. A title no excerpt has stays as it was.
pub fn linkify(answer: &str, used: &[Excerpt]) -> String {
    let mut out = String::with_capacity(answer.len());
    let mut rest = answer;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let title = after[..end].trim();
        match used.iter().find(|excerpt| excerpt.title.trim() == title) {
            Some(excerpt) => {
                let id = excerpt.id.strip_prefix("note:").unwrap_or(&excerpt.id);
                out.push_str(&format!(
                    "[{}]({}{id})",
                    title,
                    kanso_core::links::NOTE_LINK_PREFIX
                ));
            }
            None => out.push_str(&rest[start..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Searches `store` for `question` and returns the excerpts to send, or
/// nothing when the notes don't cover it.
pub fn retrieve(store: &Store, question: &str) -> kanso_store::Result<Vec<Excerpt>> {
    let terms = question_terms(question);
    if terms.is_empty() || question.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut lists = Vec::new();
    for term in &terms {
        let query = Query {
            clauses: vec![Clause {
                negated: false,
                field: None,
                value: term.clone(),
            }],
        };
        let params = SearchParams {
            query,
            limit: Some(TEXT_RESULTS),
            ..SearchParams::default()
        };
        if let Ok(docs) = store.search(&params) {
            lists.push(docs.into_iter().map(|doc| doc.id).collect());
        }
    }
    if let MeaningSearch::Found(found) = store.search_by_meaning(question, MEANING_RESULTS)? {
        lists.push(found.into_iter().map(|note| note.id).collect());
    }
    let ranked = fuse(&lists, MAX_NOTES * 4);
    let tags: HashMap<String, String> = store
        .tags()?
        .into_iter()
        .map(|tag| (tag.id, tag.name))
        .collect();
    let mut candidates = Vec::new();
    for id in ranked {
        // Fail closed: a note whose lock can't be checked, or is locked even
        // though this session has it unlocked, is never read or sent.
        if store.note_is_locked(&id).unwrap_or(true) {
            continue;
        }
        let Some(note) = store.note(&id)? else {
            continue;
        };
        candidates.push(Candidate {
            id: note.id.clone(),
            title: note.title.clone(),
            body: note.body.clone(),
            tags: note
                .tags
                .iter()
                .map(|id| tags.get(id).cloned().unwrap_or_else(|| id.clone()))
                .collect(),
            locked: false,
            trashed: note.is_trashed(),
        });
    }
    Ok(select(question, &candidates, MAX_NOTES, BUDGET))
}

#[cfg(test)]
mod tests {
    use super::{
        Candidate, best_text, eligible, fuse, is_exclude_tag, linkify, question_terms, retrieve,
        select, stream_answer,
    };
    use kanso_ai::{AiError, ChatProvider, Event, Excerpt, Prompt};
    use kanso_core::DocType;
    use kanso_store::{Origin, Put, Store};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn candidate(id: &str, body: &str, tags: &[&str], locked: bool, trashed: bool) -> Candidate {
        Candidate {
            id: id.to_owned(),
            title: id.to_owned(),
            body: body.to_owned(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            locked,
            trashed,
        }
    }

    /// The IDs the exclusions leave, for a property's expectation.
    fn available_ids(candidates: &[Candidate]) -> Vec<&str> {
        candidates
            .iter()
            .filter(|candidate| eligible(candidate))
            .map(|candidate| candidate.id.as_str())
            .collect()
    }

    #[test]
    fn the_exclusion_tag_is_written_any_way() {
        for tag in ["ai-exclude", "#ai-exclude", " AI-EXCLUDE ", "AI-Exclude"] {
            assert!(is_exclude_tag(tag), "{tag}");
        }
        for tag in ["ai", "exclude", "ai-excluded", "notes"] {
            assert!(!is_exclude_tag(tag), "{tag}");
        }
    }

    #[test]
    fn questions_lose_their_common_words() {
        assert_eq!(
            question_terms("What did we decide about the pricing in May?"),
            ["decide", "pricing", "may"]
        );
        assert_eq!(question_terms("is it?"), ["is it?"]);
        assert!(question_terms("").is_empty());
    }

    #[test]
    fn fusion_prefers_a_note_in_several_lists() {
        let merged = fuse(
            &[
                vec!["note:a".into(), "note:b".into()],
                vec!["note:b".into(), "note:c".into()],
            ],
            3,
        );
        assert_eq!(merged, ["note:b", "note:a", "note:c"]);
    }

    #[test]
    fn the_matching_passage_comes_before_the_opening() {
        let text = best_text(
            "A note about the garden.\n\nWe chose the blue paint for the kitchen.",
            &["paint".to_owned()],
            400,
        );
        assert!(text.starts_with("We chose the blue paint"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Whatever the notes and their flags, a locked, trashed or
        /// `#ai-exclude` note is never in what a question sends.
        #[test]
        fn prop_excluded_notes_are_never_sent(
            question in "\\PC{0,80}",
            rows in prop::collection::vec(
                ("note:[a-z]{1,6}", "\\PC{0,300}", any::<bool>(), any::<bool>(), any::<bool>()),
                0..14,
            ),
        ) {
            let candidates: Vec<Candidate> = rows
                .iter()
                .map(|(id, body, locked, trashed, exclude)| Candidate {
                    id: id.clone(),
                    title: id.clone(),
                    body: body.clone(),
                    tags: if *exclude { vec!["ai-exclude".to_owned()] } else { vec![] },
                    locked: *locked,
                    trashed: *trashed,
                })
                .collect();
            let chosen = select(&question, &candidates, 8, 20_000);
            let allowed = available_ids(&candidates);
            for excerpt in &chosen {
                prop_assert!(allowed.contains(&excerpt.id.as_str()), "{} was sent", excerpt.id);
            }
            for (candidate, (_, _, locked, trashed, exclude)) in candidates.iter().zip(&rows) {
                if *locked || *trashed || *exclude {
                    prop_assert!(!chosen.iter().any(|excerpt| excerpt.id == candidate.id));
                }
            }
        }

        /// Every excerpt fits the budget, is labelled with its note, and
        /// comes from its own note's words.
        #[test]
        fn prop_every_excerpt_is_labeled_and_inside_the_budget(
            question in "\\PC{0,80}",
            rows in prop::collection::vec(("note:[a-z]{1,6}", "\\PC{0,500}"), 0..10),
            budget in 500usize..30_000,
        ) {
            let candidates: Vec<Candidate> = rows
                .iter()
                .map(|(id, body)| candidate(id, body, &[], false, false))
                .collect();
            let chosen = select(&question, &candidates, 8, budget);
            let total: usize = chosen.iter().map(|excerpt| excerpt.text.chars().count()).sum();
            prop_assert!(total <= budget);
            for excerpt in &chosen {
                let candidate = candidates.iter().find(|c| c.id == excerpt.id).unwrap();
                prop_assert_eq!(&excerpt.title, &candidate.title);
                prop_assert!(!excerpt.text.trim().is_empty());
                prop_assert!(!excerpt.id.is_empty());
            }
        }
    }

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn note(store: &Store, title: &str, body: &str, tags: Vec<String>) -> String {
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book, "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// A provider that says what it was given, without a network.
    struct Fake {
        events: Vec<Event>,
    }

    impl ChatProvider for Fake {
        fn stream(&self, _: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            Ok(Box::new(self.events.clone().into_iter()))
        }
    }

    #[test]
    fn an_answer_streams_in_and_stops_at_done() {
        let provider = Fake {
            events: vec![
                Event::Text("Your ".into()),
                Event::Text("notes.".into()),
                Event::Done,
                Event::Text(" Not this.".into()),
            ],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let mut said = String::new();
        stream_answer(&provider, &prompt, &AtomicBool::new(false), |event| {
            if let Event::Text(text) = event {
                said.push_str(&text);
            }
        });
        assert_eq!(said, "Your notes.");
    }

    #[test]
    fn cancelling_stops_the_answer() {
        let provider = Fake {
            events: vec![
                Event::Text("one".into()),
                Event::Text("two".into()),
                Event::Done,
            ],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let cancel = AtomicBool::new(false);
        let mut said = String::new();
        stream_answer(&provider, &prompt, &cancel, |event| {
            if let Event::Text(text) = event {
                said.push_str(&text);
                cancel.store(true, Ordering::Relaxed);
            }
        });
        assert_eq!(said, "one");
    }

    #[test]
    fn a_provider_error_is_the_last_event() {
        let provider = Fake {
            events: vec![Event::Error("offline".into()), Event::Text("late".into())],
        };
        let prompt = Prompt {
            system: String::new(),
            user: String::new(),
        };
        let mut events = Vec::new();
        stream_answer(&provider, &prompt, &AtomicBool::new(false), |event| {
            events.push(event)
        });
        assert_eq!(events, [Event::Error("offline".into())]);
    }

    #[test]
    fn a_cited_note_becomes_a_link_to_it() {
        let used = vec![
            excerpt_for("Pricing", "The yearly plan is 40 dollars."),
            excerpt_for("Launch", "It starts in May."),
        ];
        assert_eq!(
            linkify("See [[Pricing]] and [[Launch]].", &used),
            "See [Pricing](inkdrop://note/pricing) and [Launch](inkdrop://note/launch)."
        );
        assert_eq!(
            linkify("See [[Someone else's note]].", &used),
            "See [[Someone else's note]]."
        );
    }

    fn excerpt_for(title: &str, text: &str) -> Excerpt {
        Excerpt {
            id: format!("note:{}", title.to_lowercase()),
            title: title.to_owned(),
            text: text.to_owned(),
        }
    }

    proptest! {
        /// A cited title links to the note it names, and a title no excerpt
        /// has is left as it was. Linking an already linked answer changes
        /// nothing.
        #[test]
        fn prop_linkify_links_only_known_titles(
            before in "[^\\[\\]]{0,80}",
            after in "[^\\[\\]]{0,80}",
            title in "[A-Za-z][A-Za-z ]{0,20}",
            unknown in "[A-Za-z][A-Za-z ]{0,20}",
        ) {
            prop_assume!(title.trim() != unknown.trim());
            let used = vec![excerpt_for(&title, "words")];
            let answer = format!("{before}[[{title}]]{after}[[{unknown}]]");
            let linked = linkify(&answer, &used);
            let id = title.to_lowercase();
            let linked_title = format!("[{}](inkdrop://note/{id})", title.trim());
            let unknown_title = format!("[[{unknown}]]");
            prop_assert!(linked.contains(&linked_title));
            prop_assert!(linked.contains(&unknown_title));
            prop_assert_eq!(linkify(&linked, &used), linked.clone());
            if !answer.contains("[[") {
                prop_assert_eq!(linked, answer);
            }
        }
    }

    #[test]
    fn a_locked_note_is_never_read_for_a_question() {
        let (_dir, store) = open();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let locked = store
            .put(
                DocType::Note,
                json!({"title": "Secret pricing", "body": "The secret pricing is 99.",
                    "bookId": book, "kansoLock": {"salt": "abc"}}),
                Put::Upsert,
                Origin::Sync,
            )
            .unwrap()
            .id;
        let found = retrieve(&store, "What is the secret pricing?").unwrap();
        assert!(found.iter().all(|excerpt| excerpt.id != locked));
        // The note is still there, still locked: nothing was changed.
        assert!(store.note_is_locked(&locked).unwrap());
    }

    #[test]
    fn retrieval_finds_a_note_and_leaves_out_the_excluded() {
        let (_dir, store) = open();
        let kept = note(
            &store,
            "Pricing",
            "We decided the yearly plan costs 40 dollars.",
            vec![],
        );
        let excluded_tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        note(
            &store,
            "Secret pricing",
            "The pricing is elsewhere.",
            vec![excluded_tag],
        );
        let trashed = note(&store, "Old pricing", "The old pricing.", vec![]);
        store.delete(&trashed, Origin::Ui).unwrap();
        let found = retrieve(&store, "What did we decide about pricing?").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, kept);
        assert!(found[0].text.contains("yearly plan"));
        assert!(
            retrieve(&store, "Where are the car keys?")
                .unwrap()
                .is_empty()
        );
    }
}
