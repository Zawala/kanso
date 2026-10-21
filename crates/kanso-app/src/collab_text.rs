//! The pure side of shared notes in the editor: where carets and
//! selections go when someone else's edits are merged in, where other
//! people's cursors and comments sit in the text, and the words the
//! editor and the Share sheet use.
//!
//! Every offset is a UTF-8 byte offset, as the engine's splices and the
//! text editor's selections both count.

use std::ops::Range;

use kanso_collab::{CommentView, Placement, RemoteCursor, Role, Splice, remap};

use crate::collab_model::RemoteMark;

/// The share a note's `kansoShare` names: its ID, this account's role and
/// the owner's username. `None` for a note that isn't shared.
pub fn share_mark(
    extra: &serde_json::Map<String, serde_json::Value>,
) -> Option<(String, Role, String)> {
    let mark = extra.get(kanso_core::model::SHARE_FIELD)?.as_object()?;
    let share_id = mark.get("shareId")?.as_str()?.to_owned();
    let role = mark
        .get("role")
        .and_then(|r| r.as_str())
        .and_then(Role::parse)
        .unwrap_or(Role::View);
    let owner = mark
        .get("ownerName")
        .and_then(|o| o.as_str())
        .unwrap_or_default()
        .to_owned();
    Some((share_id, role, owner))
}

/// The largest char boundary of `text` at or before `at`.
pub fn floor_boundary(text: &str, at: usize) -> usize {
    let mut at = at.min(text.len());
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Where a selection goes through `splices` (applied in order): each end
/// follows its own text, so a caret before the edits stays put and one
/// after them moves with what it is next to. `anchor` and `head` keep their
/// order, so a selection made backwards stays backwards.
pub fn map_selection(anchor: usize, head: usize, splices: &[Splice]) -> (usize, usize) {
    (remap(anchor, splices), remap(head, splices))
}

/// Vim's cursor and shown Visual selection after a merge: they follow
/// their text as the editor's selection does, and in Normal mode the
/// cursor stays on a character, never past the end of its line.
pub fn map_vim(
    head: usize,
    shown: Option<Range<usize>>,
    normal: bool,
    splices: &[Splice],
    text: &str,
) -> (usize, Option<Range<usize>>) {
    let head = floor_boundary(text, remap(head, splices));
    let head = if normal {
        crate::vim::clamp_normal(text, head)
    } else {
        head
    };
    let shown = shown.map(|range| {
        let start = floor_boundary(text, remap(range.start, splices));
        let end = floor_boundary(text, remap(range.end, splices));
        start.min(end)..start.max(end)
    });
    (head, shown)
}

/// Other people's cursors as the editor draws them: in the text (clamped
/// to it, on character boundaries), the selection in order.
pub fn remote_marks(cursors: &[RemoteCursor], text: &str) -> Vec<RemoteMark> {
    cursors
        .iter()
        .map(|cursor| {
            let anchor = floor_boundary(text, cursor.anchor);
            let head = floor_boundary(text, cursor.head);
            RemoteMark {
                name: cursor.name.clone(),
                caret: head,
                selection: anchor.min(head)..anchor.max(head),
                colour: cursor_colour(cursor.colour),
            }
        })
        .collect()
}

/// Which of the theme's cursor colours an index from the engine picks:
/// the same one on every device, whatever the index.
pub fn cursor_colour(index: usize) -> usize {
    index % crate::theme::CURSOR_COLOURS.len()
}

/// The text the open, anchored comments are on, to tint in the editor:
/// clamped to `text` and on character boundaries; empty ranges left out.
pub fn comment_ranges(comments: &[CommentView], text: &str) -> Vec<Range<usize>> {
    comments
        .iter()
        .filter(|view| !view.comment.resolved)
        .filter_map(|view| match &view.placement {
            Placement::Anchored(range) => {
                let start = floor_boundary(text, range.start);
                let end = floor_boundary(text, range.end.max(range.start));
                (start < end).then_some(start..end)
            }
            Placement::Orphaned(_) => None,
        })
        .collect()
}

/// What the editor's footer says about a shared note: that it is shared,
/// the role, whether it can be changed here, and whether it is live.
pub fn share_footer(role: Role, connected: bool, ended: bool) -> String {
    if ended {
        return "Private copy · No longer shared".into();
    }
    let mut parts = vec!["Shared", role.label()];
    if !role.may_edit() {
        parts.push("Read only");
    }
    parts.push(if connected { "Live" } else { "Offline" });
    parts.join(" · ")
}

/// A fingerprint's groups of digits, to set apart on screen.
pub fn fingerprint_groups(fingerprint: &str) -> Vec<String> {
    fingerprint.split_whitespace().map(str::to_owned).collect()
}

/// "Can edit, until 28 May" and the like, for an invite link.
pub fn link_label(role: Role, days_left: i64) -> String {
    match days_left {
        ..=0 => format!("{}, expired", role.label()),
        1 => format!("{}, for 1 more day", role.label()),
        n => format!("{}, for {n} more days", role.label()),
    }
}

/// Whole days from `now_ms` until `at_ms`, rounded up.
pub fn days_until(at_ms: i64, now_ms: i64) -> i64 {
    let left = at_ms - now_ms;
    if left <= 0 {
        0
    } else {
        (left + 86_399_999) / 86_400_000
    }
}

/// The Comments tab's two groups, as indices into `comments`: those whose
/// words are still in the text, in text order, then those whose words were
/// deleted ("Orphaned"), by where their words were. Earlier comments first
/// where two start at the same place.
pub fn comment_groups(comments: &[CommentView]) -> (Vec<usize>, Vec<usize>) {
    let mut anchored = Vec::new();
    let mut orphaned = Vec::new();
    for (ix, view) in comments.iter().enumerate() {
        match &view.placement {
            Placement::Anchored(range) => anchored.push((range.start, view.comment.created_at, ix)),
            Placement::Orphaned(at) => orphaned.push((*at, view.comment.created_at, ix)),
        }
    }
    anchored.sort();
    orphaned.sort();
    (
        anchored.into_iter().map(|(_, _, ix)| ix).collect(),
        orphaned.into_iter().map(|(_, _, ix)| ix).collect(),
    )
}

/// What clicking a comment selects in the editor: its words, or a caret
/// where an orphaned comment's words were; inside `text`, on character
/// boundaries.
pub fn comment_target(placement: &Placement, text: &str) -> Range<usize> {
    match placement {
        Placement::Anchored(range) => {
            let start = floor_boundary(text, range.start);
            start..floor_boundary(text, range.end.max(start))
        }
        Placement::Orphaned(at) => {
            let at = floor_boundary(text, *at);
            at..at
        }
    }
}

/// A comment's words in runs, each marked when it is an @mention of one of
/// `members` (usernames): `@` at the start or after a non-letter, then the
/// name, as the engine finds mentions. The runs cover `body` in order.
pub fn mention_runs(body: &str, members: &[String]) -> Vec<(Range<usize>, bool)> {
    let mut runs = Vec::new();
    let mut plain = 0;
    for (i, _) in body.match_indices('@') {
        if i < plain {
            continue;
        }
        let after_word = body[..i]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric());
        if after_word {
            continue;
        }
        let name: &str = {
            let rest = &body[i + 1..];
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-')))
                .unwrap_or(rest.len());
            rest[..end].trim_end_matches(['-', '_'])
        };
        if name.is_empty()
            || !members
                .iter()
                .any(|member| member.eq_ignore_ascii_case(name))
        {
            continue;
        }
        if plain < i {
            runs.push((plain..i, false));
        }
        let end = i + 1 + name.len();
        runs.push((i..end, true));
        plain = end;
    }
    if plain < body.len() {
        runs.push((plain..body.len(), false));
    }
    runs
}

/// The Comments tab's summary: "2 open · 1 resolved"; empty with none.
pub fn comments_summary(comments: &[CommentView]) -> String {
    let resolved = comments.iter().filter(|v| v.comment.resolved).count();
    let open = comments.len() - resolved;
    let mut parts = Vec::new();
    if open > 0 {
        parts.push(format!("{open} open"));
    }
    if resolved > 0 {
        parts.push(format!("{resolved} resolved"));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_collab::{Anchor, Comment, apply_splices, splices};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn view(id: &str, placement: Placement, resolved: bool, created_at: i64) -> CommentView {
        CommentView {
            comment: Comment {
                id: id.into(),
                author: "a".into(),
                author_name: "user1".into(),
                anchor: Anchor {
                    start: String::new(),
                    end: String::new(),
                    quote: "q".into(),
                },
                body: "b".into(),
                mentions: Vec::new(),
                resolved,
                created_at,
                edited_at: None,
                seq: 0,
            },
            placement,
            pending: false,
            mine: false,
        }
    }

    /// Text with multi-byte characters, so offsets fall inside them.
    fn text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop::sample::select(vec!['a', 'b', ' ', '\n', 'é', '😀', '中']),
            0..40,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    #[test]
    fn a_caret_keeps_its_place_through_someone_elses_edit() {
        // "Hello world" → "Oh, Hello brave world": the caret after "world"
        // ends after "world" still; one inside "Hello" stays in it.
        let before = "Hello world";
        let after = "Oh, Hello brave world";
        let edits = splices(before, after);
        assert_eq!(map_selection(11, 11, &edits), (21, 21));
        assert_eq!(map_selection(2, 2, &edits).0, 6);
        // A backwards selection stays backwards.
        let (anchor, head) = map_selection(11, 6, &edits);
        assert!(anchor > head);
    }

    #[test]
    fn vim_stays_on_a_character_in_normal_mode() {
        // The line's last character was deleted under the cursor.
        let before = "abc\ndef";
        let after = "ab\ndef";
        let edits = splices(before, after);
        let (head, _) = map_vim(2, None, true, &edits, after);
        assert_eq!(head, 1, "on the `b`, not on the line's end");
        let (head, shown) = map_vim(2, Some(0..3), false, &edits, after);
        assert_eq!((head, shown), (2, Some(0..2)));
    }

    #[test]
    fn open_anchored_comments_are_tinted() {
        let comments = vec![
            view("late", Placement::Anchored(10..12), false, 1),
            view("gone", Placement::Orphaned(3), false, 5),
            view("done", Placement::Anchored(0..1), true, 3),
            view("past", Placement::Anchored(14..40), false, 3),
        ];
        assert_eq!(
            comment_ranges(&comments, "0123456789abcdef"),
            [10..12, 14..16]
        );
    }

    #[test]
    fn a_share_mark_names_the_share() {
        let extra = serde_json::json!({"kansoShare": {"shareId": "s1", "role": "comment", "owner": "a", "ownerName": "user2"}});
        assert_eq!(
            share_mark(extra.as_object().unwrap()),
            Some(("s1".into(), Role::Comment, "user2".into()))
        );
        let odd = serde_json::json!({"kansoShare": {"shareId": "s1", "role": "boss"}});
        assert_eq!(share_mark(odd.as_object().unwrap()).unwrap().1, Role::View);
        assert_eq!(share_mark(&serde_json::Map::new()), None);
    }

    #[test]
    fn footer_and_link_words() {
        assert_eq!(
            share_footer(Role::Edit, true, false),
            "Shared · Can edit · Live"
        );
        assert_eq!(
            share_footer(Role::View, false, false),
            "Shared · Can view · Read only · Offline"
        );
        assert_eq!(
            share_footer(Role::Owner, true, true),
            "Private copy · No longer shared"
        );
        assert_eq!(link_label(Role::Comment, 1), "Can comment, for 1 more day");
        assert_eq!(days_until(86_400_001, 0), 2);
        assert_eq!(days_until(5, 9), 0);
        assert_eq!(
            fingerprint_groups("01234 56789 00000 11111 22222 33333").len(),
            6
        );
    }

    #[test]
    fn comments_group_in_text_order_then_orphaned() {
        let comments = vec![
            view("late", Placement::Anchored(10..12), false, 1),
            view("gone", Placement::Orphaned(3), false, 5),
            view("first", Placement::Anchored(0..1), true, 3),
            view("same-place-older", Placement::Anchored(10..11), false, 0),
        ];
        assert_eq!(comment_groups(&comments), (vec![2, 3, 0], vec![1]));
        let (anchored, _) = comment_groups(&comments);
        let ids: Vec<&str> = anchored
            .iter()
            .map(|&i| comments[i].comment.id.as_str())
            .collect();
        assert_eq!(ids, ["first", "same-place-older", "late"]);
        assert_eq!(comments_summary(&comments), "3 open · 1 resolved");
        assert_eq!(comments_summary(&[]), "");
        assert_eq!(comment_target(&Placement::Orphaned(40), "abc"), 3..3);
        assert_eq!(comment_target(&Placement::Anchored(1..2), "é"), 0..2);
    }

    #[test]
    fn mentions_of_members_stand_out() {
        let members = vec!["user2".to_owned(), "ann-b".to_owned()];
        let runs = |body: &str| -> Vec<(String, bool)> {
            mention_runs(body, &members)
                .into_iter()
                .map(|(r, m)| (body[r].to_owned(), m))
                .collect()
        };
        assert_eq!(
            runs("Ask @User2, and @ann-b_."),
            [
                ("Ask ".into(), false),
                ("@User2".into(), true),
                (", and ".into(), false),
                ("@ann-b".into(), true),
                ("_.".into(), false),
            ]
        );
        // Not a member, or inside a word: plain.
        assert_eq!(
            runs("mail me@user2 or @nobody"),
            [("mail me@user2 or @nobody".into(), false)]
        );
        assert!(mention_runs("", &members).is_empty());
    }

    proptest! {
        #![proptest_config(config(256))]

        /// Every comment is in exactly one group; the anchored ones in text
        /// order, the orphaned ones in the order their words were.
        #[test]
        fn prop_comment_groups_cover_every_comment(specs in prop::collection::vec((0usize..60, 0usize..10, any::<bool>(), 0i64..5), 0..12)) {
            let comments: Vec<CommentView> = specs
                .iter()
                .enumerate()
                .map(|(i, (start, len, orphaned, at))| {
                    let placement = if *orphaned {
                        Placement::Orphaned(*start)
                    } else {
                        Placement::Anchored(*start..start + len)
                    };
                    view(&format!("c{i}"), placement, false, *at)
                })
                .collect();
            let (anchored, orphaned) = comment_groups(&comments);
            let mut all: Vec<usize> = anchored.iter().chain(&orphaned).copied().collect();
            all.sort_unstable();
            prop_assert_eq!(all, (0..comments.len()).collect::<Vec<_>>());
            let start = |ix: usize| match &comments[ix].placement {
                Placement::Anchored(r) => r.start,
                Placement::Orphaned(at) => *at,
            };
            for pair in anchored.windows(2) {
                prop_assert!(matches!(comments[pair[0]].placement, Placement::Anchored(_)));
                prop_assert!(start(pair[0]) <= start(pair[1]));
            }
            for pair in orphaned.windows(2) {
                prop_assert!(start(pair[0]) <= start(pair[1]));
            }
            for &ix in &orphaned {
                prop_assert!(matches!(comments[ix].placement, Placement::Orphaned(_)));
            }
        }

        /// What a click selects is inside the text, on character boundaries,
        /// and a caret for an orphaned comment.
        #[test]
        fn prop_a_comment_click_selects_inside_the_text(body in text(), start in 0usize..200, len in 0usize..20, orphaned in any::<bool>()) {
            let placement = if orphaned { Placement::Orphaned(start) } else { Placement::Anchored(start..start + len) };
            let range = comment_target(&placement, &body);
            prop_assert!(range.start <= range.end && range.end <= body.len());
            prop_assert!(body.is_char_boundary(range.start) && body.is_char_boundary(range.end));
            if orphaned {
                prop_assert!(range.is_empty());
            }
        }

        /// The runs cover the words exactly, in order, and every marked run
        /// is `@` and a member's name.
        #[test]
        fn prop_mention_runs_cover_the_words(words in prop::collection::vec(prop::sample::select(vec!["@user2", "@USER2", "@ann", "x@user2", "@", "@nobody", " ", "é", "-", "_", "hi"]), 0..12)) {
            let body: String = words.concat();
            let members = vec!["user2".to_owned(), "ann".to_owned()];
            let runs = mention_runs(&body, &members);
            let mut at = 0;
            for (range, mention) in &runs {
                prop_assert_eq!(range.start, at);
                prop_assert!(range.start < range.end);
                at = range.end;
                if *mention {
                    let name = &body[range.start + 1..range.end];
                    prop_assert!(body[range.clone()].starts_with('@'));
                    prop_assert!(members.iter().any(|m| m.eq_ignore_ascii_case(name)));
                }
            }
            prop_assert_eq!(at, body.len());
        }

        /// The splices take the editor's text to the merged text, and the
        /// caret ends on a character boundary of it, after the same text it
        /// was after when that text survived.
        #[test]
        fn prop_a_caret_follows_its_text(before in text(), after in text(), at in 0usize..200) {
            let at = floor_boundary(&before, at);
            let edits = splices(&before, &after);
            let mut applied = before.clone();
            apply_splices(&mut applied, &edits);
            prop_assert_eq!(&applied, &after);
            let (anchor, head) = map_selection(at, at, &edits);
            prop_assert_eq!(anchor, head);
            prop_assert!(head <= after.len());
            prop_assert!(after.is_char_boundary(head));
            // Untouched text before every edit keeps the caret where it was.
            let first = edits.iter().map(|s| s.at).min().unwrap_or(usize::MAX);
            if at <= first && edits.iter().all(|s| s.at >= at && !(s.at == at && s.delete == 0)) {
                prop_assert_eq!(head, at);
            }
        }

        /// A selection's ends never cross: mapped, the start stays at or
        /// before the end.
        #[test]
        fn prop_selections_keep_their_order(before in text(), after in text(), a in 0usize..200, b in 0usize..200) {
            let (a, b) = (floor_boundary(&before, a.min(b)), floor_boundary(&before, a.max(b)));
            let edits = splices(&before, &after);
            let (start, end) = map_selection(a, b, &edits);
            prop_assert!(start <= end);
        }

        /// Vim's Normal cursor lands on a character of its line (or the
        /// start of an empty one), and its Visual selection in the text.
        #[test]
        fn prop_vim_positions_stay_in_the_text(before in text(), after in text(), head in 0usize..200, a in 0usize..200, b in 0usize..200, normal in any::<bool>()) {
            let edits = splices(&before, &after);
            let head = floor_boundary(&before, head);
            let shown = Some(floor_boundary(&before, a.min(b))..floor_boundary(&before, a.max(b)));
            let (head, shown) = map_vim(head, shown, normal, &edits, &after);
            prop_assert!(after.is_char_boundary(head));
            if normal {
                prop_assert_eq!(head, crate::vim::clamp_normal(&after, head));
            }
            let shown = shown.unwrap();
            prop_assert!(shown.start <= shown.end && shown.end <= after.len());
        }

        /// Every remote cursor lands in the text, its caret at one end of
        /// its selection, its colour one of the theme's.
        #[test]
        fn prop_remote_marks_fit_the_text(body in text(), cursors in prop::collection::vec((0usize..200, 0usize..200, 0usize..64), 0..6)) {
            let cursors: Vec<RemoteCursor> = cursors
                .into_iter()
                .enumerate()
                .map(|(i, (anchor, head, colour))| RemoteCursor {
                    account: format!("a{i}"),
                    device: "d".into(),
                    name: format!("user{i}"),
                    anchor,
                    head,
                    colour,
                    seen_at: 0,
                })
                .collect();
            for (mark, cursor) in remote_marks(&cursors, &body).iter().zip(&cursors) {
                prop_assert!(mark.selection.end <= body.len());
                prop_assert!(body.is_char_boundary(mark.caret));
                prop_assert!(mark.caret == mark.selection.start || mark.caret == mark.selection.end);
                prop_assert!(mark.colour < crate::theme::CURSOR_COLOURS.len());
                prop_assert_eq!(mark.colour, cursor.colour % crate::theme::CURSOR_COLOURS.len());
            }
        }

        /// Tints are exactly the open, anchored comments with words left,
        /// each inside the text on character boundaries.
        #[test]
        fn prop_comment_tints_fit_the_text(body in text(), specs in prop::collection::vec((0usize..60, 0usize..10, any::<bool>(), any::<bool>()), 0..10)) {
            let comments: Vec<CommentView> = specs
                .iter()
                .enumerate()
                .map(|(i, (start, len, orphaned, resolved))| {
                    let placement = if *orphaned {
                        Placement::Orphaned(*start)
                    } else {
                        Placement::Anchored(*start..start + len)
                    };
                    view(&format!("c{i}"), placement, *resolved, 0)
                })
                .collect();
            let tints = comment_ranges(&comments, &body);
            for range in &tints {
                prop_assert!(range.start < range.end && range.end <= body.len());
                prop_assert!(body.is_char_boundary(range.start) && body.is_char_boundary(range.end));
            }
            let open = comments.iter().filter(|v| !v.comment.resolved && matches!(v.placement, Placement::Anchored(_))).count();
            prop_assert!(tints.len() <= open);
        }
    }
}
