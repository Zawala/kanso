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
    match item[3..].chars().next() {
        None | Some(' ') | Some('\t') => Some(checked),
        _ => None,
    }
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
}
