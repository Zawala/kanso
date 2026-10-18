//! Property tests for kanso-core. CI pins the seed with `PROPTEST_RNG_SEED`.

use kanso_core::id::{is_valid_id, with_prefix};
use kanso_core::query::{contains_ci, find_ci};
use kanso_core::tasks::{count_tasks, toggle_task};
use kanso_core::{Clause, DocType, Field, NoteFacts, Query, Rev, new_id};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn gen_field() -> impl Strategy<Value = Option<Field>> {
    prop::option::of(prop::sample::select(Field::ALL.to_vec()))
}

/// Values with the characters the syntax treats specially: quotes, colons,
/// backslashes, dashes and (unicode) whitespace.
fn gen_value() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 :\"\\\\_\\-é漢\t\u{3000}]{1,10}"
}

fn gen_clause() -> impl Strategy<Value = Clause> {
    (any::<bool>(), gen_field(), gen_value()).prop_map(|(negated, field, value)| Clause {
        negated,
        field,
        value,
    })
}

fn gen_query() -> impl Strategy<Value = Query> {
    prop::collection::vec(gen_clause(), 0..6).prop_map(|clauses| Query { clauses })
}

#[derive(Debug)]
struct OwnedFacts {
    title: String,
    body: String,
    book_id: String,
    book_name: Option<String>,
    status: String,
    tag_ids: Vec<String>,
    tag_names: Vec<String>,
}

impl OwnedFacts {
    fn facts(&self) -> NoteFacts<'_> {
        NoteFacts {
            title: &self.title,
            body: &self.body,
            book_id: &self.book_id,
            book_name: self.book_name.as_deref(),
            status: &self.status,
            tag_ids: &self.tag_ids,
            tag_names: &self.tag_names,
        }
    }
}

fn gen_facts() -> impl Strategy<Value = OwnedFacts> {
    (
        "[a-zA-Z :]{0,20}",
        "[a-zA-Z :\n]{0,60}",
        prop::sample::select(vec!["book:a", "book:b", "trash"]),
        prop::option::of(prop::sample::select(vec!["Alpha", "beta"])),
        prop::sample::select(vec!["none", "active", "onHold"]),
        prop::collection::vec(prop::sample::select(vec!["tag:x", "tag:y"]), 0..3),
    )
        .prop_map(
            |(title, body, book_id, book_name, status, tag_ids)| OwnedFacts {
                title,
                body,
                book_id: book_id.to_owned(),
                book_name: book_name.map(str::to_owned),
                status: status.to_owned(),
                tag_names: tag_ids.iter().map(|t| t.replace("tag:", "T")).collect(),
                tag_ids: tag_ids.into_iter().map(str::to_owned).collect(),
            },
        )
}

/// Markdown-ish bodies without code fences, so appended tasks always count.
fn gen_fence_free_body() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "- [ ] open",
            "- [x] done",
            "  * [X] nested",
            "1. [ ] numbered",
            "plain line",
            "- not a task",
            "- [] broken",
            "",
        ]),
        0..12,
    )
    .prop_map(|lines| lines.join("\n"))
}

fn gen_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        "[a-z ]{0,8}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::btree_map("[a-z]{1,4}", inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

proptest! {
    #![proptest_config(config())]

    /// Round-trip: formatting a query and parsing it back gives the same query.
    #[test]
    fn prop_query_format_parse_roundtrip(query in gen_query()) {
        prop_assert_eq!(Query::parse(&query.to_string()), query);
    }

    /// Refinement + idempotence: any input parses to non-empty clauses, and
    /// re-parsing its formatted form changes nothing.
    #[test]
    fn prop_parse_any_input_is_stable(input in "\\PC{0,40}") {
        let parsed = Query::parse(&input);
        prop_assert!(parsed.clauses.iter().all(|c| !c.value.is_empty()));
        prop_assert_eq!(Query::parse(&parsed.to_string()), parsed);
    }

    /// Metamorphic: negating a clause flips whether a note matches it.
    #[test]
    fn prop_negation_complements(clause in gen_clause(), note in gen_facts()) {
        let facts = note.facts();
        let plain = Query { clauses: vec![Clause { negated: false, ..clause.clone() }] };
        let negated = Query { clauses: vec![Clause { negated: true, ..clause }] };
        prop_assert_ne!(plain.matches(&facts), negated.matches(&facts));
    }

    /// Metamorphic: adding a clause can only narrow the matches (AND semantics).
    #[test]
    fn prop_more_clauses_narrow(query in gen_query(), extra in gen_clause(), note in gen_facts()) {
        let facts = note.facts();
        let mut narrowed = query.clone();
        narrowed.clauses.push(extra);
        prop_assert!(!narrowed.matches(&facts) || query.matches(&facts));
    }

    /// Oracle: the ASCII fast path agrees with lowercase-then-contains.
    #[test]
    fn prop_contains_ci_matches_oracle(h in "[a-zA-Z0-9 ]{0,30}", n in "[a-zA-Z0-9 ]{0,4}") {
        prop_assert_eq!(contains_ci(&h, &n), h.to_lowercase().contains(&n.to_lowercase()));
    }

    /// Oracle: for ASCII text `find_ci` agrees with `contains_ci`, and what
    /// it finds is the first position where the needle sits, in any case.
    #[test]
    fn prop_find_ci_finds_the_first_ascii_match(h in "[aAbB ]{0,30}", n in "[aAbB ]{0,4}") {
        let found = find_ci(&h, &n);
        prop_assert_eq!(found.is_some(), contains_ci(&h, &n));
        if let Some(at) = found {
            prop_assert!(h[at..at + n.len()].eq_ignore_ascii_case(&n));
            prop_assert!((0..at).all(|i| !h[i..].to_lowercase().starts_with(&n.to_lowercase())));
        }
    }

    /// Oracle (brute force): in any text, `find_ci` returns the first
    /// character boundary where the folded rest of the text starts with the
    /// folded needle, and nothing when there is none.
    #[test]
    fn prop_find_ci_matches_a_brute_force_search(h in "[aAéÉßΣσςİi K\u{212A}k]{0,16}", n in "[aAéÉΣσİi K\u{212A}k]{1,3}") {
        let fold = |s: &str| s.chars().flat_map(char::to_lowercase).collect::<String>();
        let needle = fold(&n);
        let expected = h.char_indices().map(|(i, _)| i).find(|&i| fold(&h[i..]).starts_with(&needle));
        prop_assert_eq!(find_ci(&h, &n), expected);
    }

    /// Invariant: the terms for a field are exactly the clauses that can
    /// match it, in order: free text and that field's own, never negated.
    #[test]
    fn prop_terms_for_a_field_are_its_positive_text_clauses(query in gen_query(), field in prop::sample::select(vec![Field::Title, Field::Body])) {
        let terms: Vec<&str> = query.terms_for(field).collect();
        let mut expected = Vec::new();
        for clause in &query.clauses {
            let can_match = clause.field.is_none() || clause.field == Some(field);
            if can_match && !clause.negated {
                expected.push(clause.value.as_str());
            }
        }
        prop_assert_eq!(terms, expected);
    }

    /// Metamorphic: a substring of the title always matches as free text, in any case.
    #[test]
    fn prop_title_substring_matches(note in gen_facts(), start in 0usize..20, len in 1usize..6) {
        let chars: Vec<char> = note.title.chars().collect();
        prop_assume!(start + len <= chars.len());
        let needle: String = chars[start..start + len].iter().collect::<String>().to_uppercase();
        prop_assume!(!needle.trim().is_empty());
        let query = Query { clauses: vec![Clause::text(needle)] };
        prop_assert!(query.matches(&note.facts()));
    }

    /// Round-trip: revisions format and parse back unchanged.
    #[test]
    fn prop_rev_roundtrip(generation in 1u64.., hash in "[0-9a-f]{32}") {
        let rev = Rev { generation, hash };
        prop_assert_eq!(rev.to_string().parse::<Rev>().unwrap(), rev);
    }

    /// Monotonicity + determinism: each write bumps the generation by one and
    /// equal content hashes equally.
    #[test]
    fn prop_rev_next_is_monotonic(content in gen_json(), generation in 1u64..1_000_000) {
        let prev = Rev { generation, hash: "0".repeat(32) };
        let next = Rev::next(Some(&prev), &content);
        prop_assert_eq!(next.generation, generation + 1);
        prop_assert_eq!(next.hash.len(), 32);
        prop_assert_eq!(Rev::next(Some(&prev), &content), next);
    }

    /// Invariant + metamorphic: checked tasks never exceed the total, and
    /// appending an open task adds exactly one unchecked task.
    #[test]
    fn prop_task_counts(body in gen_fence_free_body()) {
        let before = count_tasks(&body);
        prop_assert!(before.checked <= before.total);
        let after = count_tasks(&format!("{body}\n- [ ] appended"));
        prop_assert_eq!(after.total, before.total + 1);
        prop_assert_eq!(after.checked, before.checked);
    }

    /// Round-trip: toggling a task twice restores the body (with `[X]`
    /// normalized to `[x]`), and one toggle moves the checked count by one.
    #[test]
    fn prop_toggle_task_twice_is_identity(body in gen_fence_free_body(), pick in any::<prop::sample::Index>()) {
        let counts = count_tasks(&body);
        prop_assume!(counts.total > 0);
        let index = pick.index(counts.total as usize);
        let once = toggle_task(&body, index).unwrap();
        let after = count_tasks(&once);
        prop_assert_eq!(after.total, counts.total);
        prop_assert_eq!(after.checked.abs_diff(counts.checked), 1);
        let twice = toggle_task(&once, index).unwrap();
        prop_assert_eq!(twice.replace("[X]", "[x]"), body.replace("[X]", "[x]"));
    }

    /// Refinement: generated IDs are valid IDs of their type.
    #[test]
    fn prop_new_ids_are_valid(doc_type in prop::sample::select(DocType::ALL.to_vec())) {
        let id = new_id(doc_type);
        prop_assert!(is_valid_id(doc_type, &id));
        prop_assert_eq!(DocType::of_id(&id), Some(doc_type));
    }

    /// Idempotence: adding a prefix twice is the same as once.
    #[test]
    fn prop_with_prefix_is_idempotent(suffix in "[a-zA-Z0-9_:-]{1,12}") {
        let once = with_prefix(DocType::Tag, &suffix);
        prop_assert_eq!(with_prefix(DocType::Tag, &once), once.clone());
        prop_assert!(once.starts_with("tag:"));
    }
}
