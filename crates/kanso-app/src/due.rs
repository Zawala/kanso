//! Due dates on tasks (`- [ ] Pay rent @2026-10-25`, `@2026-10-25T09:00`)
//! where the note is written and read: a small chip in the preview
//! ("Tomorrow", "Fri 25 Oct", in a warm red once overdue), a quiet tint on
//! the token in the editor, and `@` autocomplete for the date in a task.
//!
//! The preview is handed each task's due token written as a
//! `kanso-due:` link (as wiki-links become ordinary links), which
//! [`DueChip`] draws. The chip works out its words when it is drawn, so
//! "Today" stays true.

use std::ops::Range;
use std::time::Duration;

use gpui_kit::base::input::{Rope, RopeExt as _, TextDecoration, TextDecorationCollection};
use gpui_kit::base::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    markdown_ast,
};
use gpui_kit::component::input::EditorState;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::{Date, DateTime};
use kanso_core::links::code_ranges;
use kanso_core::tasks::{Due, TaskGroup, due_label, due_spans, task_group, task_items};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, TextEdit};

use crate::theme::Colors;
use crate::zoom::px;

/// The scheme of the links due dates become in the preview.
pub const DUE_SCHEME: &str = "kanso-due:";
/// Marks a done task's date in its link.
const DONE_SUFFIX: &str = ";done";
/// How long the editor waits after typing stops to tint new dates.
const MARK_DELAY: Duration = Duration::from_millis(150);

/// How a due date is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Upcoming, or without anything to say.
    Plain,
    /// Due today.
    Today,
    /// Past, and the task isn't done.
    Overdue,
    /// The task is done.
    Done,
}

/// How a task's date reads at local time `now`. Pure.
pub fn tone(due: Due, checked: bool, now: DateTime) -> Tone {
    match task_group(Some(due), checked, now) {
        TaskGroup::Done => Tone::Done,
        TaskGroup::Overdue => Tone::Overdue,
        TaskGroup::Today => Tone::Today,
        TaskGroup::Upcoming | TaskGroup::NoDate => Tone::Plain,
    }
}

/// The chip's words at local time `now`: "Tomorrow", "Fri 25 Oct 09:00",
/// and "Overdue · Yesterday" for a past date on a task not done. Pure.
pub fn chip_label(due: Due, checked: bool, now: DateTime) -> String {
    let label = due_label(due, now.date);
    if tone(due, checked, now) == Tone::Overdue {
        format!("Overdue · {label}")
    } else {
        label
    }
}

/// "Sunday, 25 October 2026", with "at 09:00" for a time: the chip's
/// tooltip.
pub fn long_label(due: Due) -> String {
    let day = due.date.long_label();
    match due.time {
        Some((hour, minute)) => format!("Due {day} at {hour:02}:{minute:02}"),
        None => format!("Due {day}"),
    }
}

/// The note's Markdown with each task's due date, outside code, written as
/// a `kanso-due:` link for [`DueChip`] to draw. Text without dates comes
/// back as it is. Pure.
pub fn rewrite_for_preview(text: &SharedString) -> SharedString {
    if !text.contains('@') {
        return text.clone();
    }
    let spans = due_spans(text);
    if spans.is_empty() {
        return text.clone();
    }
    let code = code_ranges(text);
    let mut out = String::with_capacity(text.len() + spans.len() * 24);
    let mut last = 0;
    for span in spans {
        if code
            .iter()
            .any(|c| c.start < span.range.end && span.range.start < c.end)
        {
            continue;
        }
        out.push_str(&text[last..span.range.start]);
        let token = &text[span.range.clone()];
        out.push('[');
        out.push_str(token);
        out.push_str("](");
        out.push_str(DUE_SCHEME);
        out.push_str(&token[1..]);
        if span.checked {
            out.push_str(DONE_SUFFIX);
        }
        out.push(')');
        last = span.range.end;
    }
    out.push_str(&text[last..]);
    out.into()
}

/// The due date (and whether its task is done) a `kanso-due:` link holds.
pub fn parse_due_url(url: &str) -> Option<(Due, bool)> {
    let rest = url.strip_prefix(DUE_SCHEME)?;
    let (date, checked) = match rest.strip_suffix(DONE_SUFFIX) {
        Some(date) => (date, true),
        None => (rest, false),
    };
    Some((Due::parse(date)?, checked))
}

/// What a chip shows.
struct Chip {
    due: Due,
    checked: bool,
}

/// Draws the links due dates become: a small rounded chip with the date in
/// words, tinted by how near it is.
pub struct DueChip;

impl DueChip {
    pub const NAME: &'static str = "kanso-due-chip";
}

/// A chip's text and fill colors for `tone`.
pub fn chip_colors(tone: Tone, colors: &Colors) -> (Hsla, Hsla) {
    let tinted = |rgb: crate::theme::Rgb, alpha: f32| {
        let color = Hsla::from(rgb);
        (color, color.opacity(alpha))
    };
    match tone {
        Tone::Overdue => tinted(colors.danger, 0.16),
        Tone::Today => tinted(colors.link, 0.16),
        Tone::Plain => (colors.text_secondary.into(), colors.fill.into()),
        Tone::Done => (
            colors.text_muted.into(),
            Hsla::from(colors.fill).opacity(0.6),
        ),
    }
}

impl MarkdownPlugin for DueChip {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else {
            return None;
        };
        let (due, checked) = parse_due_url(&link.url)?;
        let source = cx.node_source(node).unwrap_or_default().to_owned();
        Some(
            MarkdownNode::new(Self::NAME, Chip { due, checked })
                .text(format!("@{}", due.format()))
                .markdown(source),
        )
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        context: &InlineRenderContext,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        let chip = node.data::<Chip>()?;
        let now = crate::journal::now(cx);
        let colors = Colors::get(cx);
        let tone = tone(chip.due, chip.checked, now);
        let (text, fill) = chip_colors(tone, colors);
        let style = context.text_style();
        let size = context.font_size() * 0.8;
        let at = node.source_range().map_or(0, |r| r.start);
        let tooltip = long_label(chip.due);
        let element = div()
            .id(ElementId::NamedInteger(Self::NAME.into(), at as u64))
            .h(context.line_height())
            .flex()
            .items_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .px(size * 0.5)
                    .h(size * 1.55)
                    .rounded(size * 0.45)
                    .bg(fill)
                    .font_family(style.font_family.clone())
                    .font_weight(FontWeight::MEDIUM)
                    .text_size(size)
                    .line_height(size * 1.2)
                    .text_color(text)
                    .when(tone == Tone::Done, |el| el.line_through())
                    .child(
                        gpui_kit::component::Icon::new(gpui_kit::assets::IconName::Calendar)
                            .size(size * 0.95)
                            .text_color(text),
                    )
                    .child(chip_label(chip.due, chip.checked, now)),
            )
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(tooltip.clone()).build(window, cx)
            });
        Some(InlineElement::new(element))
    }
}

// MARK: The editor

/// The tints the editor gives the due dates in its text, kept as text
/// decorations that follow edits, and worked out again a moment after
/// typing stops.
#[derive(Default)]
pub struct DueMarks {
    collection: Option<TextDecorationCollection>,
    task: Option<Task<()>>,
    /// How many dates are tinted now.
    count: usize,
}

/// The tint of each due date in `body` at local time `now`. Pure.
pub fn marks(body: &str, now: DateTime) -> Vec<(Range<usize>, Tone)> {
    due_spans(body)
        .into_iter()
        .map(|span| (span.range, tone(span.due, span.checked, now)))
        .collect()
}

impl DueMarks {
    /// Tints the dates in `body`'s text now.
    pub fn refresh(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.task = None;
        let text = body.read(cx).value();
        let now = crate::journal::now(cx);
        let colors = Colors::get(cx);
        let decorations: Vec<TextDecoration> = marks(&text, now)
            .into_iter()
            .map(|(range, tone)| {
                let (color, fill) = chip_colors(tone, colors);
                TextDecoration::new(
                    range,
                    HighlightStyle {
                        color: Some(color),
                        background_color: Some(fill),
                        ..Default::default()
                    },
                )
            })
            .collect();
        self.count = decorations.len();
        match &self.collection {
            Some(collection) => collection.set(decorations, cx),
            None if decorations.is_empty() => {}
            None => {
                self.collection = Some(body.update(cx, |state, cx| {
                    state.create_decorations_collection(decorations, cx)
                }));
            }
        }
    }

    /// How many dates are tinted now.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Tints the dates once typing pauses. The decorations already there
    /// follow the edits meanwhile.
    pub fn refresh_soon<V: 'static>(
        &mut self,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut DueMarks,
        cx: &mut App,
    ) {
        self.task = Some(cx.spawn(async move |cx| {
            cx.background_executor().timer(MARK_DELAY).await;
            cx.update(|cx| {
                if let Some(owner) = owner.upgrade() {
                    owner.update(cx, |owner, cx| {
                        let marks = pick(owner);
                        marks.task = None;
                        marks.refresh(&body, cx);
                    });
                }
            });
        }));
    }
}

// MARK: Autocomplete

/// What `@` autocomplete is completing in a task, from the text of the
/// caret's line up to the caret: where the `@` is and what was typed after
/// it. `None` outside a task, or once the typing can't be a date. Pure.
pub fn date_query(before_caret: &str) -> Option<(usize, &str)> {
    let at = before_caret.rfind('@')?;
    if at > 0 && !before_caret[..at].ends_with([' ', '\t', '(']) {
        return None;
    }
    let query = &before_caret[at + 1..];
    if query.len() > 16
        || !query
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':')
    {
        return None;
    }
    // A task: its box, then some words, before the `@`.
    let line = format!("{}x", &before_caret[..at]);
    (!task_items(&line).is_empty()).then_some((at, query))
}

/// The dates offered for `query` on `today`, as `(label, date)`: today,
/// tomorrow, next week (the coming Monday) and the next weekdays, those
/// whose words or date start with what was typed, then the date typed
/// itself when it is a whole one. Pure.
pub fn date_choices(query: &str, today: Date) -> Vec<(String, String)> {
    let query = query.trim().to_lowercase();
    let mut choices: Vec<(String, Date)> = vec![
        ("Today".into(), today),
        ("Tomorrow".into(), today.add_days(1)),
        (
            "Next week".into(),
            today.add_days(7 - today.weekday() as i64),
        ),
    ];
    for ahead in 2..7 {
        let day = today.add_days(ahead);
        choices.push((kanso_core::dates::WEEKDAYS[day.weekday()].to_owned(), day));
    }
    let mut out: Vec<(String, String)> = choices
        .into_iter()
        .filter(|(label, date)| {
            query.is_empty()
                || label.to_lowercase().starts_with(&query)
                || label
                    .to_lowercase()
                    .split(' ')
                    .any(|word| word.starts_with(&query))
                || date.iso().starts_with(&query)
        })
        .map(|(label, date)| {
            let words = due_label(Due { date, time: None }, today);
            let label = if words == label {
                label
            } else {
                format!("{label} · {words}")
            };
            (label, date.iso())
        })
        .collect();
    if let Some(due) = Due::parse(&query.to_uppercase().replace('t', "T"))
        && !out.iter().any(|(_, iso)| *iso == due.format())
    {
        out.push((due_label(due, today), due.format()));
    }
    out
}

/// `@` autocomplete for the date of the task the caret is in: the dates
/// [`date_choices`] offers, each replacing the `@` and what follows it.
/// `None` when the caret isn't after an `@` in a task.
pub fn completions(text: &Rope, offset: usize, cx: &App) -> Option<Vec<CompletionItem>> {
    let offset = offset.min(text.len());
    let position = text.offset_to_position(offset);
    let line_start = text.line_start_offset(position.line as usize);
    let before = text.slice(line_start..offset).to_string();
    let (at, query) = date_query(&before)?;
    let today = crate::journal::now(cx).date;
    let range = lsp_types::Range {
        start: text.offset_to_position(line_start + at),
        end: text.offset_to_position(offset),
    };
    let items = date_choices(query, today)
        .into_iter()
        .map(|(label, iso)| CompletionItem {
            label,
            detail: Some(iso.clone()),
            kind: Some(CompletionItemKind::VALUE),
            filter_text: Some(String::new()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: format!("@{iso}"),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}

#[cfg(test)]
mod tests {
    use super::{
        DUE_SCHEME, Date, DateTime, Due, Tone, chip_label, date_choices, date_query, long_label,
        marks, parse_due_url, rewrite_for_preview, tone,
    };
    use gpui_kit::SharedString;
    use proptest::prelude::*;

    fn now() -> DateTime {
        DateTime {
            date: Date::new(2026, 10, 20).unwrap(),
            hour: 10,
            minute: 0,
            second: 0,
        }
    }

    #[test]
    fn chips_read_in_words() {
        let due = |s: &str| Due::parse(s).unwrap();
        assert_eq!(chip_label(due("2026-10-21"), false, now()), "Tomorrow");
        assert_eq!(
            chip_label(due("2026-10-23T09:00"), false, now()),
            "Fri 23 Oct 09:00"
        );
        assert_eq!(
            chip_label(due("2026-10-19"), false, now()),
            "Overdue · Yesterday"
        );
        assert_eq!(chip_label(due("2026-10-19"), true, now()), "Yesterday");
        assert_eq!(tone(due("2026-10-20T09:00"), false, now()), Tone::Overdue);
        assert_eq!(tone(due("2026-10-20T11:00"), false, now()), Tone::Today);
        assert_eq!(tone(due("2026-10-19"), true, now()), Tone::Done);
        assert_eq!(
            long_label(due("2026-10-25T09:00")),
            "Due Sunday, 25 October 2026 at 09:00"
        );
    }

    #[test]
    fn the_preview_gets_links_for_due_dates() {
        let text: SharedString =
            "- [ ] Pay @2026-10-25 now\n- [x] Done @2026-10-01T09:00\n`- [ ] @2026-10-02`\n```\n- [ ] @2026-10-03\n```\nmail me@2026-10-04\n"
                .into();
        let out = rewrite_for_preview(&text);
        assert_eq!(
            out.as_ref(),
            "- [ ] Pay [@2026-10-25](kanso-due:2026-10-25) now\n- [x] Done [@2026-10-01T09:00](kanso-due:2026-10-01T09:00;done)\n`- [ ] @2026-10-02`\n```\n- [ ] @2026-10-03\n```\nmail me@2026-10-04\n"
        );
        let plain: SharedString = "no dates here".into();
        assert_eq!(rewrite_for_preview(&plain), plain);
        assert_eq!(
            parse_due_url("kanso-due:2026-10-01T09:00;done"),
            Some((Due::parse("2026-10-01T09:00").unwrap(), true))
        );
        assert_eq!(parse_due_url("kanso-due:soon"), None);
    }

    #[test]
    fn autocomplete_offers_dates_in_tasks() {
        assert_eq!(date_query("- [ ] Pay rent @"), Some((15, "")));
        assert_eq!(date_query("- [ ] Pay rent @tom"), Some((15, "tom")));
        assert_eq!(date_query("Pay rent @tom"), None, "not a task");
        assert_eq!(date_query("- [ ] mail me@x"), None);
        assert_eq!(date_query("- [ ] a @to do"), None);
        let today = now().date; // A Tuesday.
        let all = date_choices("", today);
        assert_eq!(all[0], ("Today".into(), "2026-10-20".into()));
        assert_eq!(all[1], ("Tomorrow".into(), "2026-10-21".into()));
        assert_eq!(
            all[2],
            ("Next week · Mon 26 Oct".into(), "2026-10-26".into())
        );
        assert_eq!(all.len(), 8);
        let fri = date_choices("fr", today);
        assert_eq!(
            fri,
            vec![("Friday · Fri 23 Oct".into(), "2026-10-23".into())]
        );
        let typed = date_choices("2026-12-01t08:30", today);
        assert_eq!(
            typed.last().unwrap(),
            &("Tue 1 Dec 08:30".into(), "2026-12-01T08:30".into())
        );
        assert!(date_choices("zz", today).is_empty());
    }

    proptest! {
        /// Round trip: every due date the preview gets as a link reads back
        /// from it as the same date, done or not, and the rest of the text
        /// is kept.
        #[test]
        fn prop_due_links_round_trip(
            days in 0i64..40_000,
            time in prop::option::of((0u32..24, 0u32..60)),
            checked: bool,
            words in "[a-z ]{1,10}",
        ) {
            let due = Due { date: Date::from_days(days), time };
            let mark = if checked { 'x' } else { ' ' };
            let text: SharedString = format!("- [{mark}] {words} @{}\n", due.format()).into();
            let out = rewrite_for_preview(&text);
            let url_start = out.find(DUE_SCHEME).unwrap();
            let url_end = url_start + out[url_start..].find(')').unwrap();
            prop_assert_eq!(parse_due_url(&out[url_start..url_end]), Some((due, checked)));
            let start = format!("- [{mark}] {words} [@{}](", due.format());
            prop_assert!(out.starts_with(&start));
            // The tints cover exactly the token.
            let marks = marks(&text, now());
            prop_assert_eq!(marks.len(), 1);
            prop_assert_eq!(&text[marks[0].0.clone()], format!("@{}", due.format()));
        }

        /// Every choice autocomplete offers is a real date written the way
        /// tasks carry them, and the words it shows start with what was
        /// typed (or its date does).
        #[test]
        fn prop_choices_are_dates(query in "[a-z0-9-]{0,4}", days in 0i64..40_000) {
            let today = Date::from_days(days);
            for (label, iso) in date_choices(&query, today) {
                let due = Due::parse(&iso);
                prop_assert!(due.is_some(), "{}", iso);
                let label = label.to_lowercase();
                prop_assert!(
                    query.is_empty()
                        || label.split([' ', '·']).any(|w| w.starts_with(&query))
                        || iso.starts_with(&query)
                        || Due::parse(&query).is_some(),
                    "{} for {}", label, query
                );
            }
        }
    }
}
