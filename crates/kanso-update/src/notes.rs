//! Release notes as the app shows them.
//!
//! The release page and `latest.json` carry the same changelog section
//! (cliff.toml). In the app, "Under the hood" (builds, CI, tests, docs) is
//! left out: it is there on the release page, which the sheet links to.

/// The heading of the section the app leaves out.
pub const UNDER_THE_HOOD: &str = "Under the hood";

/// `notes` without its "Under the hood" section, from that heading to the
/// next heading of the same level or above. Whitespace at the ends goes.
pub fn for_app(notes: &str) -> String {
    let mut out = Vec::new();
    let mut skipping: Option<usize> = None;
    let mut in_fence = false;
    for line in notes.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        let level = if in_fence { None } else { heading_level(line) };
        if let (Some(depth), Some(level)) = (skipping, level)
            && level <= depth
        {
            skipping = None;
        }
        if skipping.is_none()
            && let Some(level) = level
            && heading_text(line).eq_ignore_ascii_case(UNDER_THE_HOOD)
        {
            skipping = Some(level);
            continue;
        }
        if skipping.is_none() {
            out.push(line);
        }
    }
    out.join("\n").trim().to_owned()
}

/// The level of an ATX heading (`## Title` is 2).
fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    ((1..=6).contains(&hashes) && line[hashes..].starts_with(' ')).then_some(hashes)
}

fn heading_text(line: &str) -> &str {
    line.trim_start_matches('#').trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const NOTES: &str = "## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.2.0) - 2026-10-20\n\n### New\n\n- Updates install themselves ([abc1234](https://github.com/iamngoni/kanso/commit/abc1234))\n\n### Under the hood\n\n- Pin git-cliff ([def5678](https://github.com/iamngoni/kanso/commit/def5678))\n\n### Fixed\n\n- A crash\n";

    #[test]
    fn under_the_hood_is_left_out() {
        let shown = for_app(NOTES);
        assert!(shown.contains("### New"));
        assert!(shown.contains("### Fixed"));
        assert!(shown.contains("- A crash"));
        assert!(!shown.contains("Under the hood"));
        assert!(!shown.contains("git-cliff"));
        // The compare link stays.
        assert!(shown.contains("compare/v0.1.0...v0.2.0"));
    }

    #[test]
    fn a_heading_inside_code_is_text() {
        let notes = "### New\n\n```\n### Under the hood\n```\n- kept\n";
        assert_eq!(for_app(notes), notes.trim());
    }

    fn section() -> impl Strategy<Value = (String, Vec<String>)> {
        (
            prop::sample::select(vec!["New", "Improved", "Fixed", UNDER_THE_HOOD]),
            prop::collection::vec("[a-z]{1,8}( [a-z]{1,8}){0,2}", 0..4),
        )
            .prop_map(|(title, items)| (title.to_owned(), items))
    }

    proptest! {
        /// Every other section comes through whole and in order; nothing
        /// from "Under the hood" does.
        #[test]
        fn prop_only_under_the_hood_goes(sections in prop::collection::vec(section(), 0..6)) {
            let mut notes = String::from("## 1.0.0\n");
            for (title, items) in &sections {
                notes.push_str(&format!("\n### {title}\n\n"));
                for item in items {
                    let marker = if title == UNDER_THE_HOOD { "hood" } else { "kept" };
                    notes.push_str(&format!("- {marker}: {item}\n"));
                }
            }
            let shown = for_app(&notes);
            prop_assert!(!shown.contains("hood:"));
            prop_assert!(!shown.contains(UNDER_THE_HOOD));
            let kept: Vec<_> = sections.iter().filter(|(t, _)| t != UNDER_THE_HOOD).flat_map(|(_, items)| items.iter()).collect();
            let mut at = 0;
            for item in kept {
                let line = format!("- kept: {item}");
                let found = shown[at..].find(&line);
                prop_assert!(found.is_some(), "missing {}", line);
                at += found.unwrap() + line.len();
            }
            // Leaving out a section twice changes nothing more.
            prop_assert_eq!(for_app(&shown), shown);
        }
    }
}
