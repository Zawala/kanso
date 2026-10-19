//! Facts derived from a note body: task-list counts and `inkdrop://` links.

/// Counts of GitHub-style task-list items (`- [ ]` / `- [x]`) outside code fences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskCounts {
    pub total: u32,
    pub checked: u32,
}

/// Counts the task-list items in a markdown body.
pub fn count_tasks(body: &str) -> TaskCounts {
    let mut counts = TaskCounts::default();
    for line in lines_outside_fences(body) {
        if let Some(checked) = task_marker(line) {
            counts.total += 1;
            counts.checked += u32::from(checked);
        }
    }
    counts
}

/// A task-list item: where it is, whether it is done, its words and its
/// due date.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    /// The line (0-based) it is on.
    pub line: u32,
    pub checked: bool,
    /// The text after the checkbox, trimmed.
    pub text: String,
    /// `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, from an `@2026-10-25` (or
    /// `@2026-10-25T09:00`) in the text.
    pub due: Option<String>,
}

/// The task-list items in a markdown body, outside code fences, in order.
/// [`count_tasks`] counts the same items.
pub fn task_items(body: &str) -> Vec<TaskItem> {
    let mut items = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    for (ix, line) in body.lines().enumerate() {
        if let Some(open) = fence_marker(line.trim_start()) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if let Some(checked) = task_marker(line) {
            let at = line.find('[').map_or(line.len(), |at| at + 3);
            let text = line[at..].trim().to_owned();
            items.push(TaskItem {
                line: ix as u32,
                checked,
                due: due_date(&text),
                text,
            });
        }
    }
    items
}

/// The first `@YYYY-MM-DD` (with an optional `THH:MM`) in a task's text
/// that is a real date, as written without the `@`.
pub fn due_date(text: &str) -> Option<String> {
    text.match_indices('@').find_map(|(at, _)| {
        if at > 0 && !text[..at].ends_with([' ', '\t', '(']) {
            return None;
        }
        let rest = &text[at + 1..];
        let date = rest.get(..10)?;
        let b = date.as_bytes();
        let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
        if !(digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10)) {
            return None;
        }
        let year: u32 = date[0..4].parse().ok()?;
        let month: u32 = date[5..7].parse().ok()?;
        let day: u32 = date[8..10].parse().ok()?;
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        let mut end = 10;
        if let Some(time) = rest.get(10..16)
            && let [b'T', h1, h2, b':', m1, m2] = time.as_bytes()
            && [h1, h2, m1, m2].iter().all(|c| c.is_ascii_digit())
        {
            let hour = (h1 - b'0') * 10 + (h2 - b'0');
            let minute = (m1 - b'0') * 10 + (m2 - b'0');
            if hour < 24 && minute < 60 {
                end = 16;
            }
        }
        // The date must end the word.
        if rest[end..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric())
        {
            return None;
        }
        Some(rest[..end].to_owned())
    })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Toggles the `index`-th task-list item (0-based, in document order).
/// Returns `None` when the body has fewer tasks.
pub fn toggle_task(body: &str, index: usize) -> Option<String> {
    let mut seen = 0;
    let mut offset = 0;
    let mut fence: Option<(u8, usize)> = None;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
        } else if fence.is_none()
            && let Some(checked) = task_marker(line.trim_end_matches(['\n', '\r']))
        {
            if seen == index {
                let bracket = offset + line.find('[')?;
                let mut out = String::with_capacity(body.len());
                out.push_str(&body[..bracket + 1]);
                out.push(if checked { ' ' } else { 'x' });
                out.push_str(&body[bracket + 2..]);
                return Some(out);
            }
            seen += 1;
        }
        offset += line.len();
    }
    None
}

/// IDs (`note:…`) of the notes a body links to with `inkdrop://note/<id>`, in order, deduplicated.
pub fn linked_note_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for suffix in uri_suffixes(body, "inkdrop://note/") {
        let id = format!("note:{suffix}");
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// IDs (`file:…`) of the attachments a body embeds with `inkdrop://file:<id>`, deduplicated.
pub fn linked_file_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for suffix in uri_suffixes(body, "inkdrop://file:") {
        let id = format!("file:{suffix}");
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

fn uri_suffixes<'a>(body: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    body.match_indices(prefix).filter_map(move |(at, _)| {
        let rest = &body[at + prefix.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(rest.len());
        (end > 0).then(|| &rest[..end])
    })
}

fn lines_outside_fences(body: &str) -> impl Iterator<Item = &str> {
    let mut fence: Option<(u8, usize)> = None;
    body.lines().filter(move |line| {
        let trimmed = line.trim_start();
        if let Some(open) = fence_marker(trimmed) {
            fence = match fence {
                None => Some(open),
                Some((ch, len)) if open.0 == ch && open.1 >= len => None,
                other => other,
            };
            return false;
        }
        fence.is_none()
    })
}

/// `(fence char, run length)` when the line opens or closes a code fence.
fn fence_marker(trimmed: &str) -> Option<(u8, usize)> {
    let first = *trimmed.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let run = trimmed.bytes().take_while(|&b| b == first).count();
    (run >= 3).then_some((first, run))
}

/// `Some(checked)` when the line is a task-list item.
fn task_marker(line: &str) -> Option<bool> {
    let mut rest = line.trim_start();
    while let Some(after) = rest.strip_prefix('>') {
        rest = after.trim_start();
    }
    let after_marker = if let Some(after) = rest.strip_prefix(['-', '*', '+']) {
        after
    } else {
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || digits > 9 {
            return None;
        }
        rest[digits..].strip_prefix(['.', ')'])?
    };
    if !after_marker.starts_with([' ', '\t']) {
        return None;
    }
    let item = after_marker.trim_start();
    let checked = match item.get(..3)? {
        "[ ]" => false,
        "[x]" | "[X]" => true,
        _ => return None,
    };
    // Like GFM (micromark), the checkbox needs whitespace and then some text
    // after it; a bare `- [ ]` is a plain list item.
    let text = &item[3..];
    (text.starts_with([' ', '\t']) && !text.trim().is_empty()).then_some(checked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_tasks_outside_code_fences() {
        let body = "# Todo\n- [ ] one\n- [x] two\n  * [X] nested\n1. [ ] numbered\n\
                    > - [ ] quoted\n```\n- [ ] in code\n```\n- [] not a task\n-[ ] no space\n";
        assert_eq!(
            count_tasks(body),
            TaskCounts {
                total: 5,
                checked: 2
            }
        );
    }

    #[test]
    fn toggles_the_nth_task() {
        let body = "- [ ] a\n```\n- [ ] code\n```\n- [x] b\n";
        assert_eq!(
            toggle_task(body, 0).unwrap(),
            "- [x] a\n```\n- [ ] code\n```\n- [x] b\n"
        );
        assert_eq!(
            toggle_task(body, 1).unwrap(),
            "- [ ] a\n```\n- [ ] code\n```\n- [ ] b\n"
        );
        assert_eq!(toggle_task(body, 2), None);
    }

    #[test]
    fn toggling_twice_normalizes_uppercase_x() {
        let once = toggle_task("* [X] nested", 0).unwrap();
        assert_eq!(once, "* [ ] nested");
        assert_eq!(toggle_task(&once, 0).unwrap(), "* [x] nested");
    }

    #[test]
    fn extracts_links() {
        let body = "See [a](inkdrop://note/abc-1_2) and [b](inkdrop://note/abc-1_2) \
                    ![img](inkdrop://file:XyZ9) inkdrop://note/";
        assert_eq!(linked_note_ids(body), vec!["note:abc-1_2"]);
        assert_eq!(linked_file_ids(body), vec!["file:XyZ9"]);
    }

    #[test]
    fn task_items_carry_text_and_due_dates() {
        let body = "- [ ] Pay rent @2026-11-01\n```\n- [ ] no\n```\n* [x] Call @2026-10-25T09:30 ok\n- [ ] bad @2026-02-30 x@2026-01-01\n";
        let items = task_items(body);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].due.as_deref(), Some("2026-11-01"));
        assert_eq!(items[0].text, "Pay rent @2026-11-01");
        assert_eq!((items[1].line, items[1].checked), (4, true));
        assert_eq!(items[1].due.as_deref(), Some("2026-10-25T09:30"));
        assert_eq!(items[2].due, None);
        assert_eq!(due_date("@2024-02-29"), Some("2024-02-29".into()));
        assert_eq!(due_date("@2026-10-25T25:00"), None);
    }

    proptest::proptest! {
        /// The items agree with the counts, whatever the text.
        #[test]
        fn prop_items_agree_with_counts(body in "([-*] \\[[ x]\\] [a-z@0-9-]{0,12}\n|```\n|[a-z ]{0,8}\n){0,12}") {
            let items = task_items(&body);
            let counts = count_tasks(&body);
            proptest::prop_assert_eq!(items.len() as u32, counts.total);
            proptest::prop_assert_eq!(items.iter().filter(|i| i.checked).count() as u32, counts.checked);
        }
    }

    #[test]
    fn a_checkbox_without_text_is_not_a_task() {
        assert_eq!(count_tasks("- [ ]\n* [x]   \n- [ ] real\n").total, 1);
        assert_eq!(
            toggle_task("- [ ]\n- [ ] real\n", 0).as_deref(),
            Some("- [ ]\n- [x] real\n")
        );
    }
}
