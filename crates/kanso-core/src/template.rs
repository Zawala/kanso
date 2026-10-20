//! Note templates: notes in the Templates notebook whose title and body
//! may hold variables, filled in when a note is made from them.
//!
//! | Variable            | Becomes                                           |
//! |---------------------|---------------------------------------------------|
//! | `{{date}}`          | The note's date, `2026-10-20`                     |
//! | `{{time}}`          | The time it was made, `09:41`                     |
//! | `{{title}}`         | The new note's title                              |
//! | `{{date:FORMAT}}`   | The date in a format of your own, such as `{{date:%A, %e %B}}`; see [`crate::dates`] for the directives |
//! | `{{time:FORMAT}}`   | The same, for the time: `{{time:%I:%M %p}}`       |
//! | `{{cursor}}`        | Nothing; the caret starts there                   |
//!
//! Spaces inside the braces are allowed (`{{ date }}`). Anything else in
//! double braces is left exactly as typed, so text such as `{{name}}` in a
//! template survives. Values are put in once and never read again as
//! variables: a title with `{{date}}` in it stays as it is.

use crate::dates::{self, DateTime};

/// What a template's variables stand for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vars {
    /// The date `{{date}}` names (a daily note's own day), with the time
    /// `{{time}}` names.
    pub now: DateTime,
    pub title: String,
}

/// A template filled in: the text, and where `{{cursor}}` was (a byte
/// offset into `text`), if anywhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expanded {
    pub text: String,
    pub cursor: Option<usize>,
}

/// The variables Kanso knows, by their name before any `:FORMAT`.
pub const KNOWN: [&str; 4] = ["date", "time", "title", "cursor"];

/// What one variable becomes, or `None` for one Kanso doesn't know.
fn value(inner: &str, vars: &Vars) -> Option<Option<String>> {
    let inner = inner.trim();
    let (name, pattern) = match inner.split_once(':') {
        Some((name, pattern)) => (name.trim(), Some(pattern)),
        None => (inner, None),
    };
    Some(Some(match (name, pattern) {
        ("date", None) => dates::format(&vars.now, "%Y-%m-%d"),
        ("time", None) => dates::format(&vars.now, "%H:%M"),
        ("date" | "time", Some(pattern)) => dates::format(&vars.now, pattern),
        ("title", None) => vars.title.clone(),
        ("cursor", None) => return Some(None),
        _ => return None,
    }))
}

/// Fills in `template`'s variables in one pass from left to right.
pub fn expand(template: &str, vars: &Vars) -> Expanded {
    let mut text = String::with_capacity(template.len() + 32);
    let mut cursor = None;
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        text.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        // The variable ends at the first `}}`; a line break or another `{{`
        // first means these braces aren't one.
        let close = after.find("}}");
        let inner = close.map(|close| &after[..close]).filter(|inner| {
            !inner.contains('\n') && !inner.contains("{{") && !inner.trim().is_empty()
        });
        match inner.and_then(|inner| value(inner, vars).map(|v| (inner, v))) {
            Some((inner, value)) => {
                match value {
                    Some(value) => text.push_str(&value),
                    None => {
                        cursor.get_or_insert(text.len());
                    }
                }
                rest = &after[inner.len() + 2..];
            }
            None => {
                text.push_str("{{");
                rest = after;
            }
        }
    }
    text.push_str(rest);
    Expanded { text, cursor }
}

/// A note's title and body from a template's: the title is `title` when
/// given, else the template's title filled in (`{{title}}` in it is
/// empty); then the body is filled in with that title. A title is cut to
/// the longest a note takes.
pub fn instantiate(
    template_title: &str,
    template_body: &str,
    title: Option<&str>,
    now: DateTime,
) -> (String, Expanded) {
    let title = match title.map(str::trim).filter(|t| !t.is_empty()) {
        Some(title) => title.to_owned(),
        None => expand(
            template_title,
            &Vars {
                now,
                title: String::new(),
            },
        )
        .text
        .trim()
        .to_owned(),
    };
    let title: String = title
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .take(crate::model::MAX_TITLE_LEN)
        .collect();
    let body = expand(
        template_body,
        &Vars {
            now,
            title: title.clone(),
        },
    );
    (title, body)
}

/// Whether `text` has any variable Kanso fills in.
pub fn has_known_vars(text: &str) -> bool {
    let probe = Vars {
        now: DateTime::at_midnight(crate::dates::Date::from_days(0)),
        title: String::new(),
    };
    let expanded = expand(text, &probe);
    expanded.text != text || expanded.cursor.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dates::Date;
    use proptest::prelude::*;

    fn vars() -> Vars {
        Vars {
            now: DateTime {
                date: Date::new(2026, 10, 20).unwrap(),
                hour: 9,
                minute: 41,
                second: 7,
            },
            title: "Standup".into(),
        }
    }

    #[test]
    fn examples() {
        let e = expand(
            "# {{title}}\n{{ date }} at {{time}} ({{date:%A}})\n\n- {{cursor}}\n{{name}} {{}} {{date",
            &vars(),
        );
        assert_eq!(
            e.text,
            "# Standup\n2026-10-20 at 09:41 (Tuesday)\n\n- \n{{name}} {{}} {{date"
        );
        assert_eq!(e.cursor, Some(e.text.find("- \n").unwrap() + 2));
        let e = expand("{{time:%I:%M %p}} {{cursor}}{{cursor}}x", &vars());
        assert_eq!(e.text, "09:41 AM x");
        assert_eq!(e.cursor, Some(9));
        // A title isn't read again as a variable.
        let mut v = vars();
        v.title = "{{date}}".into();
        assert_eq!(expand("{{title}}", &v).text, "{{date}}");
        assert!(has_known_vars("a {{cursor}}"));
        assert!(!has_known_vars("a {{who}} b"));
    }

    /// A template: text, known variables and other variables in braces.
    fn template() -> impl Strategy<Value = String> {
        let part = prop_oneof![
            "[a-z \n#-]{0,8}",
            Just("{{date}}".to_owned()),
            Just("{{ time }}".to_owned()),
            Just("{{title}}".to_owned()),
            Just("{{cursor}}".to_owned()),
            Just("{{date:%Y/%m}}".to_owned()),
            Just("{{time:%H}}".to_owned()),
            "\\{\\{[a-z]{1,6}\\}\\}",
        ];
        prop::collection::vec(part, 0..10).prop_map(|parts| parts.concat())
    }

    /// A template that may also have stray braces.
    fn messy_template() -> impl Strategy<Value = String> {
        let part = prop_oneof![
            "[a-z \n#-]{0,8}",
            Just("{{date}}".to_owned()),
            Just("{{ time }}".to_owned()),
            Just("{{title}}".to_owned()),
            Just("{{cursor}}".to_owned()),
            Just("{{date:%Y/%m}}".to_owned()),
            Just("{{time:%H}}".to_owned()),
            "\\{\\{[a-z]{1,6}\\}\\}",
            Just("{{".to_owned()),
            Just("}}".to_owned()),
        ];
        prop::collection::vec(part, 0..10).prop_map(|parts| parts.concat())
    }

    fn known(name: &str) -> bool {
        KNOWN.contains(&name)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Invariant: no known variable is left after expanding.
        #[test]
        fn prop_no_known_variable_is_left(t in template()) {
            let e = expand(&t, &vars());
            for name in KNOWN {
                prop_assert!(!e.text.contains(&format!("{{{{{name}}}}}")), "{} -> {}", t, e.text);
            }
            prop_assert!(!e.text.contains("{{ time }}"));
            prop_assert!(!e.text.contains("{{date:"));
            prop_assert!(!e.text.contains("{{time:"));
        }

        /// Invariant: unknown variables are left exactly as typed.
        #[test]
        fn prop_unknown_variables_stay(t in template()) {
            let e = expand(&t, &vars());
            let unknown = |s: &str| -> Vec<String> {
                let mut found = Vec::new();
                let mut rest = s;
                while let Some(open) = rest.find("{{") {
                    let after = &rest[open + 2..];
                    match after.find("}}") {
                        Some(close)
                            if !after[..close].contains("{{")
                                && !after[..close].contains('\n')
                                && !after[..close].trim().is_empty() =>
                        {
                            let inner = &after[..close];
                            if !known(inner.trim().split(':').next().unwrap_or("").trim()) {
                                found.push(inner.to_owned());
                            }
                            rest = &after[close + 2..];
                        }
                        _ => rest = after,
                    }
                }
                found
            };
            let before = unknown(&t);
            prop_assert_eq!(unknown(&e.text), before);
        }

        /// Invariant: text without variables comes back unchanged, and
        /// expanding it again changes nothing.
        #[test]
        fn prop_plain_text_is_unchanged(t in "[^{}]{0,60}") {
            let e = expand(&t, &vars());
            prop_assert_eq!(&e.text, &t);
            prop_assert_eq!(e.cursor, None);
            prop_assert!(!has_known_vars(&t));
        }

        /// Invariant: the cursor, when there is one, is on a character
        /// boundary within the text.
        #[test]
        fn prop_cursor_is_in_the_text(t in messy_template()) {
            let e = expand(&t, &vars());
            if let Some(at) = e.cursor {
                prop_assert!(at <= e.text.len() && e.text.is_char_boundary(at));
            }
            prop_assert_eq!(e.cursor.is_some(), t.contains("{{cursor}}"));
        }
    }
}
