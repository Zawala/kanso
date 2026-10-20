//! Find and replace inside a note body: plain text or a regular
//! expression, optionally matching case and whole words, optionally
//! leaving fenced code blocks alone.
//!
//! Every pattern is compiled to one regular expression, so plain text and
//! a regular expression that spells it out find the same matches. Matches
//! never overlap and are never empty.

use std::ops::Range;

use regex::{Regex, RegexBuilder};

/// What to look for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern {
    pub find: String,
    pub match_case: bool,
    pub whole_word: bool,
    /// `find` is a regular expression, and `$1` or `${name}` in the
    /// replacement stand for its groups.
    pub regex: bool,
    /// Matches inside fenced code blocks (and their fences) are left alone.
    pub skip_code: bool,
}

/// Why a pattern can't be used.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    #[error("Type something to find.")]
    Empty,
    #[error("{0}")]
    Invalid(String),
}

/// A compiled [`Pattern`].
#[derive(Clone, Debug)]
pub struct Matcher {
    regex: Regex,
    expand: bool,
    skip_code: bool,
}

/// The longest regular expression accepted, compiled.
const SIZE_LIMIT: usize = 1 << 20;

impl Matcher {
    pub fn new(pattern: &Pattern) -> Result<Matcher, PatternError> {
        if pattern.find.is_empty() {
            return Err(PatternError::Empty);
        }
        let mut source = if pattern.regex {
            pattern.find.clone()
        } else {
            regex::escape(&pattern.find)
        };
        if pattern.whole_word {
            source = format!(r"\b(?:{source})\b");
        }
        let regex = RegexBuilder::new(&source)
            .case_insensitive(!pattern.match_case)
            .multi_line(true)
            .size_limit(SIZE_LIMIT)
            .build()
            .map_err(|err| PatternError::Invalid(first_line(&err.to_string())))?;
        Ok(Matcher {
            regex,
            expand: pattern.regex,
            skip_code: pattern.skip_code,
        })
    }

    /// The byte ranges of the matches in `body`, in order.
    pub fn matches(&self, body: &str) -> Vec<Range<usize>> {
        let fences = if self.skip_code {
            fence_ranges(body)
        } else {
            Vec::new()
        };
        self.regex
            .find_iter(body)
            .map(|m| m.range())
            .filter(|r| !r.is_empty())
            .filter(|r| !fences.iter().any(|f| r.start < f.end && f.start < r.end))
            .collect()
    }

    /// `body` with every match replaced by `replacement` (with `$1` and
    /// `${name}` expanded for a regular expression), and how many there
    /// were.
    pub fn replace(&self, body: &str, replacement: &str) -> (String, usize) {
        let matches = self.matches(body);
        if matches.is_empty() {
            return (body.to_owned(), 0);
        }
        let mut out = String::with_capacity(body.len());
        let mut last = 0;
        for range in &matches {
            out.push_str(&body[last..range.start]);
            if self.expand {
                // The same match again, with its groups, from its start.
                if let Some(caps) = self.regex.captures_at(body, range.start)
                    && caps.get(0).map(|m| m.range()) == Some(range.clone())
                {
                    caps.expand(replacement, &mut out);
                } else {
                    out.push_str(replacement);
                }
            } else {
                out.push_str(replacement);
            }
            last = range.end;
        }
        out.push_str(&body[last..]);
        (out, matches.len())
    }
}

fn first_line(text: &str) -> String {
    text.lines()
        .rev()
        .find(|l| l.trim_start().starts_with("error:"))
        .map(|l| {
            let l = l.trim_start().trim_start_matches("error:").trim();
            let mut s = l.to_owned();
            if let Some(c) = s.get_mut(..1) {
                c.make_ascii_uppercase();
            }
            s
        })
        .unwrap_or_else(|| "That isn't a valid regular expression.".to_owned())
}

/// The byte ranges of the fenced code blocks in `body`, fences included;
/// one left open runs to the end.
pub fn fence_ranges(body: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut open: Option<(u8, usize, usize)> = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let marker = trimmed.as_bytes().first().copied().and_then(|first| {
            if first != b'`' && first != b'~' {
                return None;
            }
            let run = trimmed.bytes().take_while(|&b| b == first).count();
            (run >= 3).then_some((first, run))
        });
        match (open, marker) {
            (None, Some((ch, len))) => open = Some((ch, len, offset)),
            (Some((ch, len, start)), Some((c, l))) if c == ch && l >= len => {
                ranges.push(start..offset + line.len());
                open = None;
            }
            _ => {}
        }
        offset += line.len();
    }
    if let Some((_, _, start)) = open {
        ranges.push(start..body.len());
    }
    ranges
}

/// A match shown in context: the line it is on (0-based), that line's
/// text, and where the match is in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchContext {
    pub line: u32,
    pub text: String,
    pub range: Range<usize>,
}

/// Each match's line and place in it, for a preview. A match running over
/// several lines is shown on its first, up to the line's end.
pub fn contexts(body: &str, matches: &[Range<usize>]) -> Vec<MatchContext> {
    let mut out = Vec::with_capacity(matches.len());
    let mut line = 0u32;
    let mut line_start = 0usize;
    let mut scanned = 0usize;
    for range in matches {
        for (ix, byte) in body.as_bytes()[scanned..range.start].iter().enumerate() {
            if *byte == b'\n' {
                line += 1;
                line_start = scanned + ix + 1;
            }
        }
        scanned = range.start;
        let line_end = body[line_start..]
            .find('\n')
            .map_or(body.len(), |e| line_start + e);
        let end = range.end.min(line_end);
        out.push(MatchContext {
            line,
            text: body[line_start..line_end].to_owned(),
            range: range.start - line_start..end - line_start,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn matcher(find: &str, regex: bool, case: bool, word: bool, skip: bool) -> Matcher {
        Matcher::new(&Pattern {
            find: find.into(),
            match_case: case,
            whole_word: word,
            regex,
            skip_code: skip,
        })
        .unwrap()
    }

    #[test]
    fn examples() {
        let body = "Cat cat catalog\n```\ncat\n```\nconcat";
        assert_eq!(
            matcher("cat", false, false, false, false)
                .matches(body)
                .len(),
            5
        );
        assert_eq!(
            matcher("cat", false, true, false, false)
                .matches(body)
                .len(),
            4
        );
        assert_eq!(
            matcher("cat", false, false, true, false)
                .matches(body)
                .len(),
            3
        );
        assert_eq!(
            matcher("cat", false, false, true, true).matches(body).len(),
            2
        );
        let (out, n) = matcher("cat", false, false, true, true).replace(body, "dog");
        assert_eq!(n, 2);
        assert_eq!(out, "dog dog catalog\n```\ncat\n```\nconcat");
        let (out, n) =
            matcher(r"(\w+)@(\w+)", true, true, false, false).replace("a@b c@d", "$2@$1");
        assert_eq!((out.as_str(), n), ("b@a d@c", 2));
        // Plain text doesn't expand `$`.
        let (out, _) = matcher("a", false, true, false, false).replace("a", "$0$1");
        assert_eq!(out, "$0$1");
        // Empty matches are never replaced.
        assert!(
            matcher("x*", true, true, false, false)
                .matches("abc")
                .is_empty()
        );
        assert_eq!(
            Matcher::new(&Pattern::default()).err(),
            Some(PatternError::Empty)
        );
        assert!(matches!(
            Matcher::new(&Pattern {
                find: "(".into(),
                regex: true,
                ..Default::default()
            }),
            Err(PatternError::Invalid(_))
        ));
        let found = 8..11;
        let ctx = contexts("one\ntwo cat\nthree", std::slice::from_ref(&found));
        assert_eq!(
            ctx,
            vec![MatchContext {
                line: 1,
                text: "two cat".into(),
                range: 4..7
            }]
        );
    }

    fn body() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                "[a-c ]{0,12}".prop_map(|s| s + "\n"),
                Just("```\n".to_owned()),
                Just("~~~~\n".to_owned()),
                "[a-c]{1,4}".prop_map(|s| format!("`{s}` ab\n")),
            ],
            0..12,
        )
        .prop_map(|lines| lines.concat())
    }

    fn fences(body: &str) -> Vec<String> {
        fence_ranges(body)
            .into_iter()
            .map(|r| body[r].to_owned())
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Plain text and a regular expression that spells it out find the
        /// same matches and replace them the same way.
        #[test]
        fn prop_regex_and_literal_agree(body in body(), find in "[a-c. *+]{1,3}", case: bool, word: bool, skip: bool, with in "[a-z$]{0,4}") {
            let literal = matcher(&find, false, case, word, skip);
            let spelled = matcher(&regex::escape(&find), true, case, word, skip);
            prop_assert_eq!(literal.matches(&body), spelled.matches(&body));
            // `$` means a group only in a regular expression; escape it there.
            let escaped = with.replace('$', "$$");
            prop_assert_eq!(literal.replace(&body, &with), spelled.replace(&body, &escaped));
        }

        /// Skipping code blocks never changes a fenced block, fences and
        /// all.
        #[test]
        fn prop_skip_code_never_alters_a_fence(body in body(), find in "[a-c`~]{1,3}", with in "[a-z`~\n]{0,4}", regex: bool) {
            let find = if regex { regex::escape(&find) } else { find };
            let m = matcher(&find, regex, true, false, true);
            let (out, _) = m.replace(&body, &with);
            // Fences are found the same way before and after, and the blocks
            // come through unchanged, unless the replacement itself writes
            // new fence markers.
            if !with.contains('`') && !with.contains('~') && !with.contains('\n') {
                prop_assert_eq!(fences(&out), fences(&body));
            }
            for range in m.matches(&body) {
                prop_assert!(fence_ranges(&body).iter().all(|f| range.end <= f.start || range.start >= f.end));
            }
        }

        /// The count replace reports is the number of matches, a
        /// replacement with the found text changes nothing, and every match
        /// is non-empty, in order and apart.
        #[test]
        fn prop_match_counts_are_consistent(body in body(), find in "[a-c]{1,2}", case: bool, word: bool, skip: bool) {
            let m = matcher(&find, false, case, word, skip);
            let found = m.matches(&body);
            let (out, n) = m.replace(&body, "XY");
            prop_assert_eq!(n, found.len());
            prop_assert_eq!(out.len() as i64, body.len() as i64 + found.iter().map(|r| 2 - r.len() as i64).sum::<i64>());
            if case {
                prop_assert_eq!(m.replace(&body, &find), (body.clone(), n));
            }
            for pair in found.windows(2) {
                prop_assert!(pair[0].end <= pair[1].start);
            }
            prop_assert!(found.iter().all(|r| !r.is_empty()));
            prop_assert_eq!(contexts(&body, &found).len(), found.len());
        }
    }
}
