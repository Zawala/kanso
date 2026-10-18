//! The middle column: search box, sort menu and the notes for the filter.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::NoteStatus;
use kanso_store::{NoteSummary, Sort};

use crate::clock::Clock;
use crate::library::{Filter, Library, LibraryEvent};
use crate::theme::Colors;

const SEARCH_DELAY: Duration = Duration::from_millis(120);
const ROW_HEIGHT: f32 = 76.;

actions!(note_list, [SelectNext, SelectPrevious]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, Some("NoteList")),
        KeyBinding::new("up", SelectPrevious, Some("NoteList")),
        KeyBinding::new("j", SelectNext, Some("NoteList")),
        KeyBinding::new("k", SelectPrevious, Some("NoteList")),
    ]);
}

/// Note operations the workspace carries out.
pub enum NoteListEvent {
    NewNote,
    TogglePinned(String),
    Trash(String),
    Restore(String),
    DeleteForever(String),
    Duplicate(String),
    /// Enter in the list or the search box: move focus to the editor.
    OpenSelected,
}

pub struct NoteList {
    library: Entity<Library>,
    pub(crate) search: Entity<InputState>,
    scroll: UniformListScrollHandle,
    focus: FocusHandle,
    search_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<NoteListEvent> for NoteList {}

impl Focusable for NoteList {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl NoteList {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search  (book:  tag:  status:  title:  -exclude)")
                .clean_on_escape()
        });
        let subscriptions = vec![
            cx.observe(&library, |_, _, cx| cx.notify()),
            // Follow the selection when it moves, but not on every reload,
            // which would pull the list back while it's being scrolled.
            cx.subscribe(&library, |this, library, event, cx| {
                if let LibraryEvent::Selected(_) = event
                    && let Some(ix) = library.read(cx).selected_index()
                {
                    this.scroll.scroll_to_item(ix, ScrollStrategy::Nearest);
                }
            }),
            cx.subscribe_in(&search, window, |this, search, event, _, cx| match event {
                InputEvent::Change => {
                    let text = search.read(cx).value().to_string();
                    let library = this.library.clone();
                    this.search_task = Some(cx.spawn(async move |_, cx| {
                        cx.background_executor().timer(SEARCH_DELAY).await;
                        library.update(cx, |library, cx| library.set_search(text, cx));
                    }));
                }
                InputEvent::PressEnter { .. } => cx.emit(NoteListEvent::OpenSelected),
                _ => {}
            }),
        ];
        NoteList {
            library,
            search,
            scroll: UniformListScrollHandle::new(),
            focus: cx.focus_handle(),
            search_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.focus(window, cx));
    }

    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    fn render_row(
        &self,
        ix: usize,
        note: &NoteSummary,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let colors = Colors::get(cx);
        // Dates read in local time: shifting both instants by the note's
        // UTC offset keeps their difference and moves the date.
        let offset_ms = i64::from(Clock::utc_offset_secs(cx, note.updated_at)) * 1000;
        let updated = relative_time(note.updated_at + offset_ms, Clock::now_ms(cx) + offset_ms);
        let id = note.id.clone();
        let library = self.library.clone();
        let focus = self.focus.clone();
        let in_trash = note.book_id == kanso_core::model::TRASH_BOOK_ID;
        let view = cx.entity();
        let menu_id = note.id.clone();
        let pinned = note.pinned;
        let title: SharedString = if note.title.trim().is_empty() {
            "Untitled".into()
        } else {
            note.title.clone().into()
        };
        v_flex()
            .id(("note", ix))
            .h(px(ROW_HEIGHT))
            .w_full()
            .px_3()
            .py_2()
            .gap_0p5()
            .border_b_1()
            .border_color(colors.divider)
            .cursor_pointer()
            .when(selected, |el| el.bg(colors.row_active))
            .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
            .child(
                h_flex()
                    .gap_1()
                    .when(pinned, |el| {
                        el.child(
                            Icon::new(IconName::StarFill)
                                .xsmall()
                                .text_color(theme.warning),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_medium()
                            .text_sm()
                            .child(title),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.text_secondary)
                    .line_clamp(2)
                    .child(note.snippet.clone()),
            )
            .child(
                h_flex()
                    .gap_2()
                    .text_xs()
                    .text_color(colors.text_muted)
                    .child(updated)
                    .when(note.status != NoteStatus::None, |el| {
                        el.child(
                            div()
                                .px_1()
                                .rounded_sm()
                                // On the selected row the chip takes the
                                // pane's color; see `Colors::fill`.
                                .bg(if selected { colors.list } else { colors.fill })
                                .text_color(colors.text_secondary)
                                .child(note.status.label()),
                        )
                    }),
            )
            .on_click(move |_, window, cx| {
                window.focus(&focus, cx);
                library.update(cx, |library, cx| library.select(Some(id.clone()), cx));
            })
            .context_menu(move |menu, _, _| {
                let emit = |event: fn(String) -> NoteListEvent| {
                    let view = view.clone();
                    let id = menu_id.clone();
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        view.update(cx, |_, cx| cx.emit(event(id.clone())));
                    }
                };
                if in_trash {
                    menu.item(PopupMenuItem::new("Restore").on_click(emit(NoteListEvent::Restore)))
                        .separator()
                        .item(
                            PopupMenuItem::new("Delete Permanently…")
                                .on_click(emit(NoteListEvent::DeleteForever)),
                        )
                } else {
                    menu.item(
                        PopupMenuItem::new(if pinned { "Unpin" } else { "Pin to Top" })
                            .on_click(emit(NoteListEvent::TogglePinned)),
                    )
                    .item(PopupMenuItem::new("Duplicate").on_click(emit(NoteListEvent::Duplicate)))
                    .separator()
                    .item(PopupMenuItem::new("Move to Trash").on_click(emit(NoteListEvent::Trash)))
                }
            })
    }
}

/// "just now", "5 min ago", "3 h ago", "2 d ago", then the date.
pub fn relative_time(then_ms: i64, now_ms: i64) -> String {
    let secs = (now_ms - then_ms).max(0) / 1000;
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86_400 => format!("{} h ago", secs / 3600),
        86_400..2_592_000 => format!("{} d ago", secs / 86_400),
        _ => format_date(then_ms),
    }
}

/// `YYYY-MM-DD` in UTC.
pub fn format_date(ms: i64) -> String {
    let days = ms.div_euclid(86_400_000);
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

impl Render for NoteList {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let title = library.filter_title();
        let count = library.notes.len();
        let (sort, descending) = (library.sort, library.descending);
        let in_trash = library.filter == Filter::Trash;
        let error = library.load_error.clone();
        let library_handle = self.library.clone();

        let sort_menu = Button::new("sort")
            .ghost()
            .xsmall()
            .icon(if descending {
                IconName::SortDescending
            } else {
                IconName::SortAscending
            })
            .tooltip("Sort")
            .dropdown_menu(move |mut menu, _, _| {
                for (label, option) in [
                    ("Date Updated", Sort::UpdatedAt),
                    ("Date Created", Sort::CreatedAt),
                    ("Title", Sort::Title),
                ] {
                    let library = library_handle.clone();
                    menu = menu.item(PopupMenuItem::new(label).checked(sort == option).on_click(
                        move |_, _, cx| {
                            let descending = option != Sort::Title;
                            library
                                .update(cx, |library, cx| library.set_sort(option, descending, cx));
                        },
                    ));
                }
                let library = library_handle.clone();
                menu.separator().item(
                    PopupMenuItem::new(if descending {
                        "Ascending"
                    } else {
                        "Descending"
                    })
                    .on_click(move |_, _, cx| {
                        library.update(cx, |library, cx| library.set_sort(sort, !descending, cx));
                    }),
                )
            });

        v_flex()
            .size_full()
            .key_context("NoteList")
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(1, cx));
            }))
            .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| {
                this.library
                    .update(cx, |library, cx| library.select_relative(-1, cx));
            }))
            // The resize handle draws the divider on the right.
            .bg(Colors::get(cx).list)
            .child(
                h_flex()
                    .h_10()
                    .px_3()
                    .gap_1()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_semibold()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(Colors::get(cx).text_muted)
                            .child(count.to_string()),
                    )
                    .child(sort_menu)
                    .when(!in_trash, |el| {
                        el.child(
                            Button::new("new-note")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .tooltip("New note (⌘N)")
                                .on_click(
                                    cx.listener(|_, _, _, cx| cx.emit(NoteListEvent::NewNote)),
                                ),
                        )
                    }),
            )
            .child(
                div()
                    .px_3()
                    .pb_2()
                    .child(Input::new(&self.search).small().cleanable(true)),
            )
            .when_some(error, |el, error| {
                el.child(
                    div()
                        .px_3()
                        .py_1()
                        .text_xs()
                        .text_color(cx.theme().danger)
                        .child(error),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        uniform_list(
                            "notes",
                            count,
                            cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                                let library = this.library.read(cx);
                                let selected = library.selected.clone();
                                let rows: Vec<(usize, NoteSummary)> = range
                                    .filter_map(|ix| library.notes.get(ix).map(|n| (ix, n.clone())))
                                    .collect();
                                rows.into_iter()
                                    .map(|(ix, note)| {
                                        let is_selected =
                                            selected.as_deref() == Some(note.id.as_str());
                                        this.render_row(ix, &note, is_selected, cx)
                                            .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .size_full()
                        .track_scroll(&self.scroll),
                    ),
            )
    }
}

impl Library {
    pub fn selected_index(&self) -> Option<usize> {
        let id = self.selected.as_ref()?;
        self.notes.iter().position(|n| &n.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_date, relative_time};
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Oracle: the date matches the proleptic Gregorian calendar as
        /// computed by stepping whole days from 1970-01-01.
        #[test]
        fn prop_format_date_matches_day_stepping(days in -800i64..80_000) {
            let (mut y, mut m, mut d) = (1970i64, 1i64, 1i64);
            let leap = |y: i64| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let len = |y: i64, m: i64| match m { 2 => if leap(y) { 29 } else { 28 }, 4 | 6 | 9 | 11 => 30, _ => 31 };
            if days >= 0 {
                for _ in 0..days {
                    d += 1;
                    if d > len(y, m) { d = 1; m += 1; if m > 12 { m = 1; y += 1; } }
                }
            } else {
                for _ in 0..-days {
                    d -= 1;
                    if d < 1 { m -= 1; if m < 1 { m = 12; y -= 1; } d = len(y, m); }
                }
            }
            prop_assert_eq!(format_date(days * 86_400_000 + 3_600_000), format!("{y:04}-{m:02}-{d:02}"));
        }

        /// Monotonicity: an older time never reads as more recent.
        #[test]
        fn prop_relative_time_is_monotonic(a in 0i64..5_000_000_000, b in 0i64..5_000_000_000) {
            let rank = |s: &str| if s == "just now" { 0 } else if s.ends_with("min ago") { 1 } else if s.ends_with(" h ago") { 2 } else if s.ends_with(" d ago") { 3 } else { 4 };
            let now = 10_000_000_000i64;
            let (older, newer) = if a < b { (a, b) } else { (b, a) };
            prop_assert!(rank(&relative_time(older, now)) >= rank(&relative_time(newer, now)));
        }
    }

    #[test]
    fn dates() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(951_782_400_000), "2000-02-29");
        assert_eq!(relative_time(1_000, 30_000), "just now");
        assert_eq!(relative_time(0, 3 * 3_600_000), "3 h ago");
    }
}
