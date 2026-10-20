//! Notebooks with a job, and daily notes.
//!
//! Three notebooks have fixed IDs so that every device finds the same one
//! without asking a server: `book:journal` holds daily notes, `book:inbox`
//! holds quick captures and `book:templates` holds templates. Each carries
//! its job in `kansoRole`, which travels with it however it is renamed.
//! They are made on first use.
//!
//! A daily note's ID is `note:daily-YYYY-MM-DD`, so two devices that make
//! the same day's note offline make the same document. Sync then merges
//! their two first versions ([`merge_daily_bodies`]) rather than letting
//! one win.

use crate::dates::Date;

/// The field of a notebook that says which job it has.
pub const ROLE_FIELD: &str = "kansoRole";
/// The start of every daily note's ID.
pub const DAILY_PREFIX: &str = "note:daily-";

/// A notebook's job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Journal,
    Inbox,
    Templates,
}

impl Role {
    pub const ALL: [Role; 3] = [Role::Journal, Role::Inbox, Role::Templates];

    /// The value of `kansoRole`.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Journal => "journal",
            Role::Inbox => "inbox",
            Role::Templates => "templates",
        }
    }

    pub fn parse(value: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.as_str() == value)
    }

    /// The notebook's fixed ID.
    pub fn book_id(self) -> &'static str {
        match self {
            Role::Journal => "book:journal",
            Role::Inbox => "book:inbox",
            Role::Templates => "book:templates",
        }
    }

    /// The name it is made with; it can be renamed freely.
    pub fn default_name(self) -> &'static str {
        match self {
            Role::Journal => "Journal",
            Role::Inbox => "Inbox",
            Role::Templates => "Templates",
        }
    }

    /// The role of the notebook `id` with fields `role_field` (its
    /// `kansoRole`): by the field, or by the fixed ID.
    pub fn of_book(id: &str, role_field: Option<&str>) -> Option<Role> {
        role_field
            .and_then(Role::parse)
            .or_else(|| Role::ALL.into_iter().find(|r| r.book_id() == id))
    }
}

/// `note:daily-2026-10-20`.
pub fn daily_id(date: Date) -> String {
    format!("{DAILY_PREFIX}{}", date.iso())
}

/// The day a daily note's ID names.
pub fn daily_date(id: &str) -> Option<Date> {
    Date::parse_iso(id.strip_prefix(DAILY_PREFIX)?)
}

/// A daily note's title: `2026-10-20`, which `[[2026-10-20]]` links to.
pub fn daily_title(date: Date) -> String {
    date.iso()
}

/// One body from two first versions of the same daily note made apart
/// (two devices offline): the lines they start and end with in common
/// stay once, and between them go `first`'s own lines, then `second`'s. A
/// side whose lines the other already has adds nothing. Every line of
/// either is a line of the result. Pure.
pub fn merge_daily_bodies(first: &str, second: &str) -> String {
    if first == second {
        return first.to_owned();
    }
    let a: Vec<&str> = first.split_inclusive('\n').collect();
    let b: Vec<&str> = second.split_inclusive('\n').collect();
    let same = |x: &&str, y: &&str| x.trim_end() == y.trim_end();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| same(x, y)).count();
    let room = a.len().min(b.len()) - prefix;
    let suffix = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(room)
        .take_while(|(x, y)| same(x, y))
        .count();
    let mid_a = a[prefix..a.len() - suffix].concat();
    let mid_b = b[prefix..b.len() - suffix].concat();
    if has_lines(first, &mid_b) {
        return first.to_owned();
    }
    if has_lines(second, &mid_a) {
        return second.to_owned();
    }
    let mut out = a[..prefix].concat();
    for part in [&mid_a, &mid_b] {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(part);
    }
    let tail = a[a.len() - suffix..].concat();
    if !tail.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&tail);
    out
}

/// Whether every line of `part` with text is a line of `text` too.
fn has_lines(text: &str, part: &str) -> bool {
    let lines: std::collections::HashSet<&str> = text.lines().map(str::trim_end).collect();
    part.lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty())
        .all(|l| lines.contains(l))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn ids_and_roles() {
        let d = Date::new(2026, 10, 20).unwrap();
        assert_eq!(daily_id(d), "note:daily-2026-10-20");
        assert_eq!(daily_date("note:daily-2026-10-20"), Some(d));
        assert_eq!(daily_date("note:daily-2026-13-20"), None);
        assert_eq!(daily_date("note:abc"), None);
        assert!(crate::id::is_valid_id(crate::DocType::Note, &daily_id(d)));
        for role in Role::ALL {
            assert!(crate::id::is_valid_id(crate::DocType::Book, role.book_id()));
            assert_eq!(Role::of_book(role.book_id(), None), Some(role));
            assert_eq!(Role::of_book("book:x", Some(role.as_str())), Some(role));
        }
        assert_eq!(Role::of_book("book:x", None), None);
    }

    #[test]
    fn merge_examples() {
        let template = "## Plan\n\n## Log\n";
        assert_eq!(merge_daily_bodies(template, template), template);
        assert_eq!(
            merge_daily_bodies(template, "## Plan\n- write\n\n## Log\n"),
            "## Plan\n- write\n\n## Log\n"
        );
        let merged = merge_daily_bodies("## Plan\n- from A\n", "## Plan\n- from B\n");
        assert_eq!(merged, "## Plan\n- from A\n- from B\n");
        let merged = merge_daily_bodies(
            "## Plan\n- from A\n\n## Log\n",
            "## Plan\n- from B\n\n## Log\n",
        );
        assert_eq!(merged, "## Plan\n- from A\n- from B\n\n## Log\n");
        assert_eq!(merge_daily_bodies("", "only B"), "only B");
        assert_eq!(merge_daily_bodies("only A", ""), "only A");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Round trip: a day's ID names it again.
        #[test]
        fn prop_daily_ids_round_trip(days in -700_000i64..2_900_000) {
            let d = Date::from_days(days);
            prop_assert_eq!(daily_date(&daily_id(d)), Some(d));
        }

        /// Invariants: nothing either side wrote is lost, and merging a
        /// version with itself changes nothing.
        #[test]
        fn prop_merge_keeps_both_sides(
            common in "([a-z#]{0,6}\n){0,3}",
            a in "([a-zA]{1,6}\n){0,3}",
            b in "([a-zB]{1,6}\n){0,3}",
        ) {
            let first = format!("{common}{a}");
            let second = format!("{common}{b}");
            let merged = merge_daily_bodies(&first, &second);
            prop_assert!(has_lines(&merged, &first), "{:?}", merged);
            prop_assert!(has_lines(&merged, &second), "{:?}", merged);
            prop_assert_eq!(merge_daily_bodies(&first, &first), first.clone());
            // Merging the result with either side again adds nothing.
            let again = merge_daily_bodies(&merged, &second);
            prop_assert_eq!(&again, &merged);
        }
    }
}
