//! The left column: the app's name and a compose button, the library
//! sections, notebooks, tags, statuses and trash, and a footer with who is
//! using it, sync, the local API and settings.

use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::NoteStatus;

use crate::clock::Clock;
use crate::editor::rgb_color;
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library};
use crate::services::{ApiState, Services, SyncState};
use crate::theme::{Colors, Rgb, muted_tag_color};
use crate::workspace::OpenSettings;

/// Things only the workspace can do (it owns dialogs and the editor).
pub enum SidebarEvent {
    NewNote,
    NewBook { parent: Option<String> },
    RenameBook(String),
    DeleteBook(String),
    RenameTag(String),
    RecolorTag(String, kanso_core::TagColor),
    DeleteTag(String),
    NewTag,
    EmptyTrash,
}

const ROW_HEIGHT: f32 = 28.;

/// What a sidebar row shows.
struct Entry {
    icon: Option<AnyElement>,
    label: SharedString,
    count: u32,
    /// How deep in the notebook tree.
    depth: usize,
}
/// Tags shown before "…" when there are many: the most used ones.
const TAG_LIMIT: usize = 10;
/// How often relative times ("Synced 3 min ago") are brought up to date.
const TICK: Duration = Duration::from_secs(30);

pub struct Sidebar {
    library: Entity<Library>,
    scroll: ScrollHandle,
    /// Whether the Status group is open; it starts closed, and opens by
    /// itself while a status is the filter.
    pub(crate) status_open: bool,
    /// Whether every tag shows, rather than the most used.
    all_tags: bool,
    user: SharedString,
    _tick: Task<()>,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Sidebar {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        cx.observe(&library, |_, _, cx| cx.notify()).detach();
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        let user = std::env::var("USER")
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "This Mac".into());
        Sidebar {
            library,
            scroll: ScrollHandle::new(),
            status_open: false,
            all_tags: false,
            user: user.into(),
            _tick: tick,
        }
    }

    fn row(
        &self,
        id: impl Into<ElementId>,
        filter: Filter,
        entry: Entry,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let Entry {
            icon,
            label,
            count,
            depth,
        } = entry;
        let selected = self.library.read(cx).filter == filter;
        let colors = Colors::get(cx);
        let library = self.library.clone();
        div()
            .id(id)
            .h(px(ROW_HEIGHT))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.))
            .mx_2()
            .pr(px(10.))
            .pl(px(10. + depth as f32 * 14.))
            .rounded(px(6.))
            .text_size(px(13.5))
            .cursor_pointer()
            .when(selected, |el| el.bg(colors.sidebar_active))
            .when(!selected, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
            .children(icon)
            .child(div().flex_1().min_w_0().truncate().child(label))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(count.to_string()),
            )
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
            })
    }

    /// A section title, with an optional control on the right.
    fn heading(
        &self,
        label: &'static str,
        action: Option<AnyElement>,
        cx: &mut Context<Self>,
    ) -> Div {
        h_flex()
            .flex_none()
            .mt(px(14.))
            .mb_0p5()
            .mx_2()
            .pl(px(10.))
            .pr_1()
            .h(px(26.))
            .text_size(px(12.5))
            .text_color(Colors::get(cx).text_secondary)
            .child(div().flex_1().child(label))
            .children(action)
    }

    fn icon(&self, name: impl Into<Icon>, cx: &mut Context<Self>) -> AnyElement {
        Icon::new(name)
            .size(px(15.))
            .text_color(Colors::get(cx).text_secondary)
            .into_any_element()
    }

    /// A small quiet icon button for a section heading.
    fn add_button(
        &self,
        id: &'static str,
        tooltip: &'static str,
        event: fn() -> SidebarEvent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        Button::new(id)
            .ghost()
            .xsmall()
            .icon(Icon::new(IconName::Plus).text_color(Colors::get(cx).text_secondary))
            .tooltip(tooltip)
            .on_click(cx.listener(move |_, _, _, cx| cx.emit(event())))
            .into_any_element()
    }

    fn render_brand(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        h_flex()
            .flex_none()
            .h(px(32.))
            .mx_2()
            .pl(px(10.))
            .pr_1()
            .gap(px(10.))
            .child(
                Icon::new(Lucide::NotebookText)
                    .size(px(16.))
                    .text_color(colors.text_secondary),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(15.))
                    .font_semibold()
                    .child("Kanso"),
            )
            .child(
                Button::new("compose")
                    .ghost()
                    .small()
                    .icon(Icon::new(Lucide::SquarePen).text_color(colors.text_secondary))
                    .tooltip("New note (⌘N)")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewNote))),
            )
    }

    fn render_tags(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let selected_tag = match &library.filter {
            Filter::Tag(id) => library.tags.iter().position(|t| &t.id == id),
            _ => None,
        };
        let counts: Vec<u32> = library.tags.iter().map(|t| t.count).collect();
        let shown = visible_tags(&counts, TAG_LIMIT, selected_tag, self.all_tags);
        let more = counts.len() > TAG_LIMIT;
        let tags: Vec<_> = shown
            .iter()
            .map(|&ix| {
                let t = &library.tags[ix];
                (t.id.clone(), t.name.clone(), t.count, t.color)
            })
            .collect();
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let chips = tags.into_iter().map(|(id, name, count, color)| {
            let view = cx.entity();
            let library = self.library.clone();
            let filter = Filter::Tag(id.clone());
            let selected = self.library.read(cx).filter == filter;
            let tag_id = id.clone();
            div()
                .id(SharedString::from(format!("tag-{id}")))
                .flex()
                .items_center()
                .h(px(24.))
                .px(px(7.))
                .rounded(px(6.))
                .border_1()
                .text_size(px(12.))
                .cursor_pointer()
                .map(|el| {
                    if selected {
                        el.bg(colors.sidebar_active)
                            .border_color(colors.sidebar_active)
                    } else {
                        el.border_color(colors.divider)
                            .hover(|el| el.bg(colors.sidebar_hover))
                    }
                })
                .child(
                    div()
                        .text_color(rgb_color(muted_tag_color(color.rgb(), dark)))
                        .child("#"),
                )
                .child(
                    div()
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        })
                        .child(name),
                )
                .child(
                    div()
                        .ml(px(5.))
                        .text_color(colors.text_muted)
                        .child(count.to_string()),
                )
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
                })
                .context_menu(move |menu, window, cx| {
                    let (rename, delete) = (view.clone(), view.clone());
                    let (a, c) = (tag_id.clone(), tag_id.clone());
                    let recolor_view = view.clone();
                    let recolor_id = tag_id.clone();
                    menu.item(PopupMenuItem::new("Rename…").on_click(move |_, _, cx| {
                        rename.update(cx, |_, cx| cx.emit(SidebarEvent::RenameTag(a.clone())));
                    }))
                    .submenu("Color", window, cx, move |mut menu, _, _| {
                        for option in kanso_core::TagColor::ALL {
                            let view = recolor_view.clone();
                            let id = recolor_id.clone();
                            menu = menu.item(PopupMenuItem::new(format!("{option:?}")).on_click(
                                move |_, _, cx| {
                                    view.update(cx, |_, cx| {
                                        cx.emit(SidebarEvent::RecolorTag(id.clone(), option))
                                    });
                                },
                            ));
                        }
                        menu
                    })
                    .separator()
                    .item(PopupMenuItem::new("Delete Tag…").on_click(move |_, _, cx| {
                        delete.update(cx, |_, cx| cx.emit(SidebarEvent::DeleteTag(c.clone())));
                    }))
                })
        });
        let toggle = more.then(|| {
            div()
                .id("more-tags")
                .flex()
                .items_center()
                .h(px(24.))
                .px(px(7.))
                .rounded(px(6.))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .cursor_pointer()
                .hover(|el| el.bg(colors.sidebar_hover))
                .child(if self.all_tags { "Fewer" } else { "…" })
                .tooltip({
                    let all = self.all_tags;
                    move |window, cx| {
                        Tooltip::new(if all {
                            "Show the most used tags"
                        } else {
                            "Show all tags"
                        })
                        .build(window, cx)
                    }
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.all_tags = !this.all_tags;
                    cx.notify();
                }))
        });
        h_flex()
            .flex_wrap()
            .flex_none()
            .gap(px(6.))
            .mx_2()
            .px(px(8.))
            .pt_1()
            .children(chips)
            .children(toggle)
    }

    fn render_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let open = self.status_open || matches!(library.filter, Filter::Status(_));
        let counts = library.counts.by_status.clone();
        let colors = Colors::get(cx);
        let header = h_flex()
            .id("status-group")
            .flex_none()
            .mt(px(14.))
            .mb_0p5()
            .mx_2()
            .pl(px(10.))
            .pr(px(8.))
            .h(px(26.))
            .rounded(px(6.))
            .text_size(px(12.5))
            .text_color(colors.text_secondary)
            .cursor_pointer()
            .hover(|el| el.bg(colors.sidebar_hover))
            .child(div().flex_1().child("Status"))
            .child(
                Icon::new(if open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(px(14.))
                .text_color(colors.text_muted),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.status_open = !this.status_open;
                cx.notify();
            }));
        let rows = open.then(|| {
            [
                NoteStatus::Active,
                NoteStatus::OnHold,
                NoteStatus::Completed,
                NoteStatus::Dropped,
            ]
            .into_iter()
            .map(|status| {
                let count = counts.get(&status).copied().unwrap_or(0);
                let icon = self.icon(status_icon(status), cx);
                self.row(
                    SharedString::from(format!("status-{}", status.as_str())),
                    Filter::Status(status),
                    Entry {
                        icon: Some(icon),
                        label: status.label().into(),
                        count,
                        depth: 0,
                    },
                    cx,
                )
            })
            .collect::<Vec<_>>()
        });
        v_flex()
            .flex_none()
            .child(header)
            .children(rows.into_iter().flatten())
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let now = Clock::now_ms(cx);
        let (api, api_tip, sync, sync_tip) = match cx.try_global::<Services>() {
            Some(services) => (
                services.api.clone(),
                services.api_status(),
                services.sync.clone(),
                services.sync_status(),
            ),
            None => (
                ApiState::Off,
                "API off".into(),
                SyncState::Off,
                "Sync off".into(),
            ),
        };
        let (tone, sync_label) = sync_line(&sync, now);
        let dot = tone_color(tone, colors);
        let api_color = match api {
            ApiState::Running(_) => colors.text_secondary,
            ApiState::Off => colors.text_muted,
            ApiState::PortInUse(_) | ApiState::Failed(_) => colors.warning,
        };
        h_flex()
            .flex_none()
            .h(px(BOTTOM_BAND_HEIGHT))
            .pl(px(12.))
            .pr(px(6.))
            .gap(px(6.))
            .border_t_1()
            .border_color(colors.divider)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded_full()
                    .bg(colors.fill)
                    .text_size(px(10.5))
                    .font_semibold()
                    .text_color(colors.text_secondary)
                    .child(initials(&self.user)),
            )
            .child(
                v_flex()
                    .id("sync-status")
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(15.))
                            .font_medium()
                            .truncate()
                            .child(self.user.clone()),
                    )
                    .child(
                        h_flex()
                            .gap(px(5.))
                            .text_size(px(11.5))
                            .line_height(px(14.))
                            .text_color(colors.text_secondary)
                            .child(div().flex_none().size(px(6.)).rounded_full().bg(dot))
                            .child(div().min_w_0().truncate().child(sync_label)),
                    )
                    .tooltip(move |window, cx| Tooltip::new(sync_tip.clone()).build(window, cx)),
            )
            .child(
                div()
                    .id("api-status")
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(20.))
                    .rounded(px(6.))
                    .child(Icon::new(Lucide::Plug).size(px(15.)).text_color(api_color))
                    .tooltip(move |window, cx| Tooltip::new(api_tip.clone()).build(window, cx)),
            )
            .child(
                div()
                    .id("settings")
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.sidebar_hover))
                    .child(
                        Icon::new(IconName::Settings)
                            .size(px(15.))
                            .text_color(colors.text_secondary),
                    )
                    .tooltip(|window, cx| Tooltip::new("Settings (⌘,)").build(window, cx))
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenSettings), cx)),
            )
    }
}

#[cfg(test)]
impl Sidebar {
    /// Every tag, as after clicking "…", scrolled into view below a long
    /// notebook tree.
    pub fn show_all_tags(&mut self, cx: &mut Context<Self>) {
        self.all_tags = true;
        self.scroll.scroll_to_bottom();
        cx.notify();
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let counts = library.counts.clone();
        let recent = library.recent;
        let books: Vec<(usize, String, String, u32)> = library
            .book_tree()
            .into_iter()
            .map(|(depth, b)| (depth, b.id.clone(), b.name.clone(), b.count))
            .collect();

        let mut list = v_flex().pb_4();
        let icon = self.icon(IconName::FileText, cx);
        list = list.child(self.row(
            "all",
            Filter::All,
            Entry {
                icon: Some(icon),
                label: "All Notes".into(),
                count: counts.all,
                depth: 0,
            },
            cx,
        ));
        let icon = self.icon(Lucide::Pin, cx);
        list = list.child(self.row(
            "pinned",
            Filter::Pinned,
            Entry {
                icon: Some(icon),
                label: "Pinned".into(),
                count: counts.pinned,
                depth: 0,
            },
            cx,
        ));
        let icon = self.icon(Lucide::Clock, cx);
        list = list.child(self.row(
            "recent",
            Filter::Recent,
            Entry {
                icon: Some(icon),
                label: "Recent".into(),
                count: recent,
                depth: 0,
            },
            cx,
        ));

        let add_book = self.add_button(
            "add-book",
            "New notebook",
            || SidebarEvent::NewBook { parent: None },
            cx,
        );
        list = list.child(self.heading("Notebooks", Some(add_book), cx));
        for (depth, id, name, count) in books {
            let view = cx.entity();
            let book_id = id.clone();
            let icon = self.icon(Lucide::Notebook, cx);
            let row = self
                .row(
                    SharedString::from(format!("book-{id}")),
                    Filter::Book(id.clone()),
                    Entry {
                        icon: Some(icon),
                        label: name.into(),
                        count,
                        depth,
                    },
                    cx,
                )
                .context_menu(move |menu, _, _| {
                    let (new, rename, delete) = (view.clone(), view.clone(), view.clone());
                    let (a, b, c) = (book_id.clone(), book_id.clone(), book_id.clone());
                    menu.item(
                        PopupMenuItem::new("New Sub-notebook").on_click(move |_, _, cx| {
                            new.update(cx, |_, cx| {
                                cx.emit(SidebarEvent::NewBook {
                                    parent: Some(a.clone()),
                                })
                            });
                        }),
                    )
                    .item(PopupMenuItem::new("Rename…").on_click(move |_, _, cx| {
                        rename.update(cx, |_, cx| cx.emit(SidebarEvent::RenameBook(b.clone())));
                    }))
                    .separator()
                    .item(
                        PopupMenuItem::new("Delete Notebook…").on_click(move |_, _, cx| {
                            delete.update(cx, |_, cx| cx.emit(SidebarEvent::DeleteBook(c.clone())));
                        }),
                    )
                });
            list = list.child(row);
        }

        let add_tag = self.add_button("add-tag", "New tag", || SidebarEvent::NewTag, cx);
        list = list
            .child(self.heading("Tags", Some(add_tag), cx))
            .child(self.render_tags(cx))
            .child(self.render_status(cx))
            .child(div().h(px(10.)));

        let trash_view = cx.entity();
        list = list.child(
            self.row(
                "trash",
                Filter::Trash,
                Entry {
                    icon: None,
                    label: "Trash".into(),
                    count: counts.trash,
                    depth: 0,
                },
                cx,
            )
            .context_menu(move |menu, _, _| {
                let view = trash_view.clone();
                menu.item(
                    PopupMenuItem::new("Empty Trash…").on_click(move |_, _, cx| {
                        view.update(cx, |_, cx| cx.emit(SidebarEvent::EmptyTrash));
                    }),
                )
            }),
        );

        let colors = Colors::get(cx);
        v_flex()
            .size_full()
            .bg(colors.sidebar)
            .text_color(colors.text)
            // The top band: the traffic lights' row, empty so the window can
            // be dragged from it.
            .child(div().flex_none().h(px(TOP_BAND_HEIGHT)))
            .child(self.render_brand(cx))
            .child(div().flex_none().h(px(6.)))
            .child(
                div()
                    .id("sidebar")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(list),
            )
            .child(self.render_footer(cx))
    }
}

fn status_icon(status: NoteStatus) -> Icon {
    match status {
        NoteStatus::Active => Lucide::CircleDot.into(),
        NoteStatus::OnHold => Lucide::CirclePause.into(),
        NoteStatus::Completed => IconName::CircleCheck.into(),
        NoteStatus::Dropped => IconName::CircleX.into(),
        NoteStatus::None => IconName::File.into(),
    }
}

/// How a status line reads at a glance, as the color of its dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Good,
    Busy,
    Quiet,
    Warning,
}

fn tone_color(tone: Tone, colors: &Colors) -> Rgb {
    match tone {
        Tone::Good => colors.success,
        Tone::Busy => colors.accent,
        Tone::Quiet => colors.text_muted,
        Tone::Warning => colors.warning,
    }
}

/// What a failed sync says in the sidebar. The reason itself (an error from
/// the network or the Keychain) only shows in the tooltip.
pub const SYNC_PROBLEMS: [&str; 4] = [
    "Sync not set up",
    "Sync token needed",
    "Offline",
    "Couldn't sync",
];

/// The sidebar's sync line: "Synced just now", "Syncing…", "Sync off", or a
/// problem in plain words.
pub fn sync_line(state: &SyncState, now_ms: i64) -> (Tone, String) {
    match state {
        SyncState::Off => (Tone::Quiet, "Sync off".into()),
        SyncState::Syncing => (Tone::Busy, "Syncing…".into()),
        SyncState::Synced {
            conflicts: 0,
            at_ms,
        } => (
            Tone::Good,
            format!("Synced {}", crate::note_list::relative_time(*at_ms, now_ms)),
        ),
        SyncState::Synced { .. } => (Tone::Warning, "Conflict detected".into()),
        SyncState::Failed(reason) => {
            let reason = reason.to_lowercase();
            let has = |words: &[&str]| words.iter().any(|w| reason.contains(w));
            let label = if has(&["worker url"]) {
                SYNC_PROBLEMS[0]
            } else if has(&["token", "keychain", "401", "403", "unauthorized"]) {
                SYNC_PROBLEMS[1]
            } else if has(&[
                "connect",
                "dns",
                "resolve",
                "timed out",
                "timeout",
                "network",
                "offline",
                "unreachable",
            ]) {
                SYNC_PROBLEMS[2]
            } else {
                SYNC_PROBLEMS[3]
            };
            (Tone::Warning, label.into())
        }
    }
}

/// Up to two capitals for an avatar: the first letter of each of the first
/// two words, so "ngoni mangudya" is "NM" and "modestnerd" is "M".
pub fn initials(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter_map(|word| word.chars().next())
        .take(2)
        .filter_map(|c| c.to_uppercase().next())
        .collect()
}

/// Which tags show, as indices in the library's (name) order: all of them
/// when asked, or else the `limit` most used plus the selected one. Ties go
/// to the earlier name.
pub fn visible_tags(
    counts: &[u32],
    limit: usize,
    selected: Option<usize>,
    all: bool,
) -> Vec<usize> {
    if all || counts.len() <= limit {
        return (0..counts.len()).collect();
    }
    let mut by_use: Vec<usize> = (0..counts.len()).collect();
    by_use.sort_by_key(|&ix| std::cmp::Reverse(counts[ix]));
    let mut shown: Vec<usize> = by_use.into_iter().take(limit).collect();
    if let Some(selected) = selected.filter(|&ix| ix < counts.len() && !shown.contains(&ix)) {
        shown.push(selected);
    }
    shown.sort_unstable();
    shown
}

#[cfg(test)]
mod tests {
    use super::{SYNC_PROBLEMS, SyncState, Tone, initials, sync_line, visible_tags};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Plain language: whatever a failed sync reports, the sidebar says
        /// one of a few fixed phrases and never the raw error.
        #[test]
        fn prop_sync_failures_read_plainly(reason in "\\PC{0,60}") {
            let (tone, label) = sync_line(&SyncState::Failed(reason.clone()), 0);
            prop_assert_eq!(tone, Tone::Warning);
            prop_assert!(SYNC_PROBLEMS.contains(&label.as_str()), "{}", label);
        }

    }

    proptest! {
        #![proptest_config(config())]

        /// A clean sync always reads "Synced …" with a good dot, however
        /// long ago; one with conflicts warns.
        #[test]
        fn prop_synced_reads_synced(at in 0i64..1_000_000_000_000, ago in 0i64..100_000_000_000, conflicts in 0usize..3) {
            let (tone, label) = sync_line(&SyncState::Synced { at_ms: at, conflicts }, at + ago);
            if conflicts == 0 {
                prop_assert_eq!(tone, Tone::Good);
                prop_assert!(label.starts_with("Synced "), "{}", label);
            } else {
                prop_assert_eq!(tone, Tone::Warning);
            }
        }

    }

    proptest! {
        #![proptest_config(config())]

        /// Initials: at most two capitals, one per word from the first two
        /// words, and none only when there is no letter or digit at all.
        #[test]
        fn prop_initials_take_the_first_letters(name in "[a-zA-Zé0-9 ._-]{0,24}") {
            let got: Vec<char> = initials(&name).chars().collect();
            let words: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
            prop_assert_eq!(got.len(), words.len().min(2));
            for (c, word) in got.iter().zip(&words) {
                prop_assert_eq!(Some(*c), word.chars().next().and_then(|f| f.to_uppercase().next()));
            }
        }

        /// The visible tags: in name order without repeats, the selected
        /// one always among them, and no hidden tag used more than a shown
        /// one (other than the selected).
        #[test]
        fn prop_visible_tags_are_the_most_used(
            counts in prop::collection::vec(0u32..20, 0..30),
            limit in 1usize..12,
            selected in prop::option::of(0usize..30),
            all in any::<bool>(),
        ) {
            let shown = visible_tags(&counts, limit, selected, all);
            prop_assert!(shown.windows(2).all(|w| w[0] < w[1]));
            prop_assert!(shown.iter().all(|&ix| ix < counts.len()));
            if all || counts.len() <= limit {
                prop_assert_eq!(shown.len(), counts.len());
            } else {
                prop_assert!(shown.len() <= limit + 1);
                if let Some(selected) = selected.filter(|&s| s < counts.len()) {
                    prop_assert!(shown.contains(&selected));
                }
                let least_shown = shown.iter().filter(|&&ix| Some(ix) != selected).map(|&ix| counts[ix]).min();
                for ix in (0..counts.len()).filter(|ix| !shown.contains(ix)) {
                    prop_assert!(least_shown.is_none_or(|least| counts[ix] <= least));
                }
                // The most used fill the limit; the selected one may come on top.
                let extra = shown.len() - limit;
                prop_assert!(extra <= 1);
                if extra == 1 {
                    prop_assert!(selected.is_some_and(|s| shown.contains(&s)));
                }
            }
        }
    }

    #[test]
    fn sync_line_examples() {
        let minute = 60_000;
        assert_eq!(
            sync_line(
                &SyncState::Synced {
                    at_ms: 0,
                    conflicts: 0
                },
                10_000
            ),
            (Tone::Good, "Synced just now".into())
        );
        assert_eq!(
            sync_line(
                &SyncState::Synced {
                    at_ms: 0,
                    conflicts: 0
                },
                3 * minute
            ),
            (Tone::Good, "Synced 3 min ago".into())
        );
        assert_eq!(
            sync_line(&SyncState::Off, 0),
            (Tone::Quiet, "Sync off".into())
        );
        assert_eq!(sync_line(&SyncState::Syncing, 0).1, "Syncing…");
        let failed = |reason: &str| sync_line(&SyncState::Failed(reason.into()), 0).1;
        assert_eq!(
            failed("set the Worker URL in Settings › Sync"),
            "Sync not set up"
        );
        assert_eq!(
            failed("no access token; set one in Settings › Sync"),
            "Sync token needed"
        );
        assert_eq!(
            failed("error sending request: connection refused"),
            "Offline"
        );
        assert_eq!(failed("the server said 500"), "Couldn't sync");
    }

    #[test]
    fn initials_examples() {
        assert_eq!(initials("modestnerd"), "M");
        assert_eq!(initials("ngoni mangudya"), "NM");
        assert_eq!(initials("jane.q.public"), "JQ");
        assert_eq!(initials(" -- "), "");
    }

    #[test]
    fn visible_tags_examples() {
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, false), vec![0, 2]);
        assert_eq!(
            visible_tags(&[5, 1, 9, 3], 2, Some(1), false),
            vec![0, 1, 2]
        );
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, true), vec![0, 1, 2, 3]);
        assert_eq!(visible_tags(&[1, 1, 1], 2, None, false), vec![0, 1]);
    }
}
