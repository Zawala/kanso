//! Comments (docs/ACCOUNTS.md › Collaboration): signed events kept outside
//! the note's text, folded into comments in log order, and placed in the
//! text by their sticky anchors.

use std::collections::BTreeMap;

use crate::doc::{CollabDoc, Field};
use crate::model::{Comment, CommentEvent, CommentOp, Placement, Role};

/// The longest comment body, in characters.
pub const MAX_BODY_CHARS: usize = 4_000;
/// The longest quote an anchor keeps, in characters.
pub const MAX_QUOTE_CHARS: usize = 280;

/// One comment event as the log delivered it, with what its envelope says
/// about its author.
#[derive(Clone, Debug)]
pub struct Logged {
    pub seq: u64,
    pub author: String,
    pub author_name: String,
    /// The author's role in the member list of the event's epoch.
    pub role: Role,
    pub event: CommentEvent,
}

/// Folds events into comments, in log order: `edit` and `delete` count from
/// the comment's author only; `resolve` and `reopen` from anyone who may
/// comment; nothing counts from a viewer, or for a comment never added (or
/// added twice).
pub fn fold(events: &[Logged]) -> BTreeMap<String, Comment> {
    let mut comments: BTreeMap<String, Comment> = BTreeMap::new();
    let mut deleted: std::collections::BTreeSet<String> = Default::default();
    // When each comment was last resolved or reopened: an older event the
    // log delivers again doesn't undo a newer one.
    let mut state_at: BTreeMap<String, i64> = BTreeMap::new();
    let mut ordered: Vec<&Logged> = events.iter().collect();
    ordered.sort_by_key(|e| e.seq);
    for logged in ordered {
        if !logged.role.may_comment() || deleted.contains(&logged.event.id) {
            continue;
        }
        let event = &logged.event;
        match event.op {
            CommentOp::Add => {
                let (Some(anchor), Some(body)) = (&event.anchor, &event.body) else {
                    continue;
                };
                if comments.contains_key(&event.id) || body.chars().count() > MAX_BODY_CHARS {
                    continue;
                }
                comments.insert(
                    event.id.clone(),
                    Comment {
                        id: event.id.clone(),
                        author: logged.author.clone(),
                        author_name: logged.author_name.clone(),
                        anchor: anchor.clone(),
                        body: body.clone(),
                        mentions: event.mentions.clone(),
                        resolved: false,
                        created_at: event.at,
                        edited_at: None,
                        seq: logged.seq,
                    },
                );
            }
            CommentOp::Edit => {
                if let (Some(comment), Some(body)) = (comments.get_mut(&event.id), &event.body)
                    && comment.author == logged.author
                    && body.chars().count() <= MAX_BODY_CHARS
                    && event.at >= comment.edited_at.unwrap_or(comment.created_at)
                {
                    comment.body = body.clone();
                    comment.edited_at = Some(event.at);
                    comment.seq = logged.seq;
                }
            }
            CommentOp::Resolve | CommentOp::Reopen => {
                let last = state_at.get(&event.id).copied().unwrap_or(i64::MIN);
                if event.at >= last
                    && let Some(comment) = comments.get_mut(&event.id)
                {
                    state_at.insert(event.id.clone(), event.at);
                    comment.resolved = event.op == CommentOp::Resolve;
                    comment.seq = logged.seq;
                }
            }
            CommentOp::Delete => {
                if comments
                    .get(&event.id)
                    .is_some_and(|c| c.author == logged.author)
                {
                    comments.remove(&event.id);
                    deleted.insert(event.id.clone());
                }
            }
        }
    }
    comments
}

/// Where a comment sits in the note's body now. Its anchor's two positions
/// span its words; when they meet and the quoted words are gone, the
/// comment is orphaned at that place. An anchor this document can't resolve
/// is orphaned at the start.
pub fn place(doc: &CollabDoc, comment: &Comment) -> Placement {
    let start = doc.resolve(&comment.anchor.start);
    let end = doc.resolve(&comment.anchor.end);
    match (start, end) {
        (Some(start), Some(end)) if start < end => Placement::Anchored(start..end),
        (Some(at), _) | (None, Some(at)) => {
            if comment.anchor.quote.is_empty() {
                Placement::Anchored(at..at)
            } else {
                Placement::Orphaned(at)
            }
        }
        (None, None) => Placement::Orphaned(0),
    }
}

/// An anchor for the byte range `range` of the body.
pub fn anchor(doc: &CollabDoc, range: std::ops::Range<usize>) -> crate::model::Anchor {
    let body = doc.text(Field::Body);
    let start = range.start.min(body.len());
    let end = range.end.clamp(start, body.len());
    let quote: String = body
        .get(start..end)
        .unwrap_or("")
        .chars()
        .take(MAX_QUOTE_CHARS)
        .collect();
    crate::model::Anchor {
        start: doc.sticky(Field::Body, start),
        end: doc.sticky(Field::Body, end),
        quote,
    }
}

/// The accounts a comment mentions: each `@name` that is a member's
/// username, as their account IDs, once each, the author left out.
pub fn mentions(body: &str, members: &[(String, String)], author: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in body.match_indices('@') {
        let before_ok = body[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if !before_ok {
            continue;
        }
        let name: String = body[i + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect::<String>()
            .to_ascii_lowercase();
        let name = name.trim_end_matches(['-', '_']);
        if let Some((account, _)) = members.iter().find(|(_, username)| username == name)
            && account != author
            && !out.contains(account)
        {
            out.push(account.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Anchor;
    use proptest::prelude::*;

    fn add(seq: u64, author: &str, role: Role, id: &str) -> Logged {
        Logged {
            seq,
            author: author.into(),
            author_name: author.into(),
            role,
            event: CommentEvent {
                id: id.into(),
                op: CommentOp::Add,
                anchor: Some(Anchor {
                    start: String::new(),
                    end: String::new(),
                    quote: "q".into(),
                }),
                body: Some(format!("by {author}")),
                mentions: vec![],
                at: seq as i64,
            },
        }
    }

    fn op(seq: u64, author: &str, role: Role, id: &str, op: CommentOp) -> Logged {
        Logged {
            seq,
            author: author.into(),
            author_name: author.into(),
            role,
            event: CommentEvent {
                id: id.into(),
                op,
                anchor: None,
                body: (op == CommentOp::Edit).then(|| format!("edited by {author}")),
                mentions: vec![],
                at: seq as i64,
            },
        }
    }

    #[test]
    fn only_the_author_edits_or_deletes_and_anyone_commenting_resolves() {
        let events = vec![
            add(1, "a", Role::Comment, "c1"),
            op(2, "b", Role::Edit, "c1", CommentOp::Edit),
            op(3, "b", Role::Edit, "c1", CommentOp::Delete),
            op(4, "b", Role::Comment, "c1", CommentOp::Resolve),
            op(5, "v", Role::View, "c1", CommentOp::Reopen),
            add(6, "v", Role::View, "c2"),
            op(7, "a", Role::Comment, "c1", CommentOp::Edit),
        ];
        let folded = fold(&events);
        assert_eq!(folded.len(), 1);
        let c1 = &folded["c1"];
        assert_eq!(c1.body, "edited by a");
        assert!(c1.resolved);
    }

    #[test]
    fn mentions_pick_members_only() {
        let members = vec![
            ("a_1".to_owned(), "ana".to_owned()),
            ("a_2".to_owned(), "bo-b".to_owned()),
        ];
        assert_eq!(
            mentions(
                "hi @ana and @bo-b, not me@ana or @carl. @Ana again",
                &members,
                "a_9"
            ),
            vec!["a_1".to_owned(), "a_2".to_owned()]
        );
        assert!(mentions("@ana", &members, "a_1").is_empty());
    }

    #[test]
    fn a_comment_whose_words_are_deleted_is_orphaned() {
        let doc = CollabDoc::from_text("", "one two three");
        let comment = Comment {
            id: "c".into(),
            author: "a".into(),
            author_name: "a".into(),
            anchor: anchor(&doc, 4..7),
            body: "b".into(),
            mentions: vec![],
            resolved: false,
            created_at: 0,
            edited_at: None,
            seq: 1,
        };
        assert_eq!(comment.anchor.quote, "two");
        assert_eq!(place(&doc, &comment), Placement::Anchored(4..7));
        doc.set_text(Field::Body, "zero one two three");
        assert_eq!(place(&doc, &comment), Placement::Anchored(9..12));
        doc.set_text(Field::Body, "zero one  three");
        assert!(matches!(place(&doc, &comment), Placement::Orphaned(9)));
    }

    proptest! {
        #![proptest_config(crate::tests::config(128))]

        /// Folding is a function of the log, not of delivery order, and a
        /// viewer's events never change it.
        #[test]
        fn prop_fold_ignores_order_and_viewers(ops in prop::collection::vec((0usize..3, 0usize..3, 0usize..6, 0usize..3), 0..30)) {
            let authors = ["a", "b", "v"];
            let ids = ["x", "y", "z"];
            let kinds = [CommentOp::Add, CommentOp::Edit, CommentOp::Resolve, CommentOp::Reopen, CommentOp::Delete, CommentOp::Add];
            let mut events = Vec::new();
            for (seq, (a, i, k, _)) in ops.iter().enumerate() {
                let role = if *a == 2 { Role::View } else { Role::Comment };
                let seq = seq as u64 + 1;
                events.push(if kinds[*k] == CommentOp::Add { add(seq, authors[*a], role, ids[*i]) } else { op(seq, authors[*a], role, ids[*i], kinds[*k]) });
            }
            let folded = fold(&events);
            let mut reversed = events.clone();
            reversed.reverse();
            prop_assert_eq!(&fold(&reversed), &folded);
            let without_viewer: Vec<Logged> = events.iter().filter(|e| e.role != Role::View).cloned().collect();
            prop_assert_eq!(&fold(&without_viewer), &folded);
            for comment in folded.values() {
                prop_assert_ne!(comment.author.as_str(), "v");
            }
        }
    }
}
