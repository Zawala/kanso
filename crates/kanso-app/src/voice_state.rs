//! Voice input's rules, apart from the microphone and the model: where it
//! stands as the button is pressed and the transcript comes back
//! ([`Phase`]), and how a transcript goes into the note ([`voice_edit`]).

use std::ops::Range;

use crate::markdown_edit::Edit;
use crate::voice_commands::dictated;

/// Where voice input stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    /// The microphone is on; `session` names this go.
    Listening { session: u64 },
    /// The microphone is off and the model is turning it into text.
    Transcribing { session: u64 },
}

/// What happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The mic button, its shortcut or menu item.
    Press,
    /// Escape, or the pill's cancel: nothing said goes in.
    Cancel,
    /// The microphone couldn't start or stopped working.
    MicFailed,
    /// A transcript came back for `session`: text, or an error.
    Transcribed { session: u64, ok: bool },
}

/// What the app does about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    /// Turn the microphone on for `session`.
    Listen(u64),
    /// Turn it off and transcribe what it heard.
    Transcribe,
    /// Turn it off, or drop what is being transcribed.
    Drop,
    /// Put the transcript in the note.
    Insert,
    /// Say what went wrong.
    ShowError,
}

impl Phase {
    /// The phase after `event`, and what to do. `next_session` names the
    /// next go (the caller counts them up).
    pub fn next(self, event: Event, next_session: u64) -> (Phase, Effect) {
        use {Effect as E, Event as Ev, Phase as P};
        match (self, event) {
            (P::Idle, Ev::Press) => (
                P::Listening {
                    session: next_session,
                },
                E::Listen(next_session),
            ),
            (P::Listening { session }, Ev::Press) => (P::Transcribing { session }, E::Transcribe),
            (P::Listening { .. } | P::Transcribing { .. }, Ev::Cancel) => (P::Idle, E::Drop),
            (P::Listening { .. }, Ev::MicFailed) => (P::Idle, E::ShowError),
            (P::Transcribing { session }, Ev::Transcribed { session: s, ok }) if s == session => {
                (P::Idle, if ok { E::Insert } else { E::ShowError })
            }
            // Pressing while transcribing waits for it; a late transcript
            // from a go that was dropped is dropped too.
            (phase, _) => (phase, E::None),
        }
    }

    pub fn is_active(self) -> bool {
        self != Phase::Idle
    }
}

/// The edit that puts `transcript` in place of `selection` in `text`, with
/// the caret after it: a space either side where it would otherwise run
/// into a word, and the first letter made small when it lands mid-sentence
/// (unless it is "I" or a word in capitals). With `commands`, spoken
/// punctuation and commands act first (`voice_commands`). Nothing when
/// nothing is left to put in.
pub fn voice_edit(
    text: &str,
    selection: Range<usize>,
    transcript: &str,
    commands: bool,
) -> Option<Edit> {
    let start = floor_boundary(text, selection.start.min(text.len()));
    let end = floor_boundary(text, selection.end.clamp(start, text.len()));
    let (out, caret) = dictated(
        &text[..start],
        text[end..].chars().next(),
        transcript,
        commands,
    )?;
    Some(Edit {
        range: start..end,
        selection: start + caret..start + caret,
        text: out,
    })
}

fn floor_boundary(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Where the live words `shown` are in `text` now, they having gone in at
/// `start`: the occurrence nearest it, for when the text around them
/// changed while someone spoke. `None` for no words, or none to be found.
pub fn find_live(text: &str, start: usize, shown: &str) -> Option<Range<usize>> {
    if shown.is_empty() {
        return None;
    }
    text.match_indices(shown)
        .map(|(at, _)| at)
        .min_by_key(|at| at.abs_diff(start))
        .map(|at| at..at + shown.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn apply(text: &str, edit: &Edit) -> String {
        format!(
            "{}{}{}",
            &text[..edit.range.start],
            edit.text,
            &text[edit.range.end..]
        )
    }

    fn put(text: &str, at: usize, transcript: &str) -> String {
        let plain = apply(text, &voice_edit(text, at..at, transcript, false).unwrap());
        let spoken = apply(text, &voice_edit(text, at..at, transcript, true).unwrap());
        assert_eq!(plain, spoken);
        plain
    }

    #[test]
    fn transcripts_go_in_as_written() {
        assert_eq!(put("", 0, " Hello there. "), "Hello there.");
        assert_eq!(
            put("Groceries:", 10, "Milk and eggs."),
            "Groceries: Milk and eggs."
        );
        assert_eq!(put("I said", 6, "Hello there."), "I said hello there.");
        assert_eq!(put("I said", 6, "I think so."), "I said I think so.");
        assert_eq!(put("We met", 6, "NASA people."), "We met NASA people.");
        assert_eq!(put("Done.", 5, "Next thing."), "Done. Next thing.");
        assert_eq!(put("- ", 2, "Buy milk"), "- Buy milk");
        assert_eq!(put("- [ ] ", 6, "Buy milk"), "- [ ] Buy milk");
        assert_eq!(put("# ", 2, "Ideas"), "# Ideas");
        assert_eq!(put("(", 1, "Aside"), "(Aside");
        assert_eq!(put("I said (", 8, "Aside"), "I said (aside");
        assert_eq!(put("one two", 4, "Three"), "one three two");
        assert_eq!(voice_edit("abc", 1..1, "  ", true), None);
        // The selection is what is replaced.
        let edit = voice_edit("say this now", 4..8, "That", true).unwrap();
        assert_eq!(apply("say this now", &edit), "say that now");
        assert_eq!(edit.selection, 8..8);
    }

    #[test]
    fn a_cancelled_go_never_inserts() {
        let (p, e) = Phase::Idle.next(Event::Press, 1);
        assert_eq!(e, Effect::Listen(1));
        let (p, e) = p.next(Event::Press, 2);
        assert_eq!(
            (p, e),
            (Phase::Transcribing { session: 1 }, Effect::Transcribe)
        );
        let (p, e) = p.next(Event::Cancel, 2);
        assert_eq!((p, e), (Phase::Idle, Effect::Drop));
        let (p, _) = p.next(Event::Press, 2);
        let (p, _) = p.next(Event::Press, 3);
        // The first go's transcript, late: not this one's.
        assert_eq!(
            p.next(
                Event::Transcribed {
                    session: 1,
                    ok: true
                },
                3
            )
            .1,
            Effect::None
        );
        assert_eq!(
            p.next(
                Event::Transcribed {
                    session: 2,
                    ok: true
                },
                3
            ),
            (Phase::Idle, Effect::Insert)
        );
    }

    fn text() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-zA-Z .,!?:\n#\\-\\[\\]()éü✓]{0,30}").unwrap()
    }

    fn event() -> impl Strategy<Value = Event> {
        prop_oneof![
            3 => Just(Event::Press),
            1 => Just(Event::Cancel),
            1 => Just(Event::MicFailed),
            2 => (1u64..6, any::<bool>()).prop_map(|(session, ok)| Event::Transcribed { session, ok }),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// The text around the selection is kept as it was, the transcript's
        /// words are all there (the first letter's case aside), and the
        /// caret lands right after them, on a character boundary.
        #[test]
        fn prop_voice_edit_keeps_the_rest_and_puts_the_words_in(
            text in text(),
            a in 0usize..40,
            b in 0usize..40,
            transcript in "[A-Za-z][a-z]{0,6}( [a-zA-Z]{1,6}){0,4}[.?]?",
            commands in any::<bool>(),
        ) {
            let (a, b) = (a.min(b), a.max(b));
            // Spoken commands ("comma", "delete that") are another test's.
            prop_assume!(
                !commands
                    || dictated("", None, &transcript, true) == dictated("", None, &transcript, false)
            );
            let edit = voice_edit(&text, a..b, &transcript, commands).unwrap();
            let out = apply(&text, &edit);
            prop_assert!(out.starts_with(&text[..edit.range.start]));
            prop_assert!(out.ends_with(&text[edit.range.end..]));
            prop_assert!(edit.range.start <= a.min(text.len()));
            prop_assert!(out.to_lowercase().contains(&transcript.to_lowercase()));
            prop_assert!(out.is_char_boundary(edit.selection.start));
            let inserted = &out[edit.range.start..edit.selection.start];
            prop_assert_eq!(inserted.trim().to_lowercase(), transcript.to_lowercase());
        }

        /// Against a model of the sessions: the microphone is on only while
        /// listening, a transcript goes in only for the go that is being
        /// transcribed, and every go ends in at most one of insert, error
        /// or drop.
        #[test]
        fn prop_each_go_ends_once_and_only_its_own_transcript_goes_in(events in proptest::collection::vec(event(), 0..40)) {
            let mut phase = Phase::Idle;
            let mut next = 1u64;
            let mut endings = std::collections::HashMap::<u64, u32>::new();
            for event in events {
                let before = phase;
                let (after, effect) = phase.next(event, next);
                if let Effect::Listen(session) = effect {
                    prop_assert_eq!(session, next);
                    next += 1;
                }
                if effect == Effect::Insert {
                    let Phase::Transcribing { session } = before else { panic!("insert while {before:?}") };
                    prop_assert_eq!(event, Event::Transcribed { session, ok: true });
                }
                if matches!(effect, Effect::Insert | Effect::ShowError | Effect::Drop) {
                    let session = match before {
                        Phase::Listening { session } | Phase::Transcribing { session } => session,
                        Phase::Idle => panic!("{effect:?} while idle"),
                    };
                    *endings.entry(session).or_default() += 1;
                    prop_assert_eq!(after, Phase::Idle);
                }
                phase = after;
            }
            prop_assert!(endings.values().all(|&n| n == 1));
        }

        /// The live words are found where they are, the nearest copy to
        /// where they went in when there are more, and never anything else.
        #[test]
        fn prop_live_words_are_found_where_they_went(
            before in "[a-z .]{0,12}",
            shown in "[a-z]{1,5}",
            between in "[a-z .]{0,12}",
            copy in any::<bool>(),
        ) {
            let text = format!("{before}{shown}{between}{}", if copy { shown.as_str() } else { "" });
            let found = find_live(&text, before.len(), &shown).unwrap();
            prop_assert_eq!(&text[found.clone()], shown.as_str());
            let nearest = text
                .match_indices(shown.as_str())
                .map(|(at, _)| at.abs_diff(before.len()))
                .min()
                .unwrap();
            prop_assert_eq!(found.start.abs_diff(before.len()), nearest);
            prop_assert_eq!(find_live(&text, 0, ""), None);
        }
    }
}
