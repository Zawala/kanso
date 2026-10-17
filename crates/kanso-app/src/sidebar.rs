//! The left column: library sections, notebooks, statuses, tags and trash.

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::NoteStatus;

use crate::editor::rgb_color;
use crate::library::{Filter, Library};

/// Things only the workspace can do (it owns dialogs and the editor).
pub enum SidebarEvent {
    NewBook { parent: Option<String> },
    RenameBook(String),
    DeleteBook(String),
    RenameTag(String),
    RecolorTag(String, kanso_core::TagColor),
    DeleteTag(String),
    NewTag,
    EmptyTrash,
}

pub struct Sidebar {
    library: Entity<Library>,
    scroll: ScrollHandle,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Sidebar {
    pub fn new(library: Entity<Library>, cx: &mut Context<Self>) -> Self {
        cx.observe(&library, |_, _, cx| cx.notify()).detach();
        Sidebar {
            library,
            scroll: ScrollHandle::new(),
        }
    }

    fn row(
        &self,
        id: impl Into<ElementId>,
        filter: Filter,
        icon: Option<AnyElement>,
        label: SharedString,
        count: Option<u32>,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let selected = self.library.read(cx).filter == filter;
        let theme = cx.theme();
        let library = self.library.clone();
        div()
            .id(id)
            .h_7()
            .flex()
            .items_center()
            .gap_2()
            .mx_2()
            .pr_2()
            .pl(px(8. + depth as f32 * 14.))
            .rounded_md()
            .text_sm()
            .cursor_pointer()
            .when(selected, |el| el.bg(theme.sidebar_accent).font_medium())
            .when(!selected, |el| {
                el.hover(|el| el.bg(theme.sidebar_accent.opacity(0.5)))
            })
            .when_some(icon, |el, icon| el.child(icon))
            .child(div().flex_1().min_w_0().truncate().child(label))
            .when_some(count, |el, count| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(count.to_string()),
                )
            })
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_filter(filter.clone(), cx));
            })
    }

    fn heading(
        &self,
        label: &'static str,
        action: Option<AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .mt_4()
            .mb_1()
            .mx_2()
            .px_2()
            .h_6()
            .text_xs()
            .font_semibold()
            .text_color(cx.theme().muted_foreground)
            .child(div().flex_1().child(label))
            .children(action)
    }

    fn icon(&self, name: IconName, cx: &mut Context<Self>) -> AnyElement {
        Icon::new(name)
            .small()
            .text_color(cx.theme().muted_foreground)
            .into_any_element()
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let counts = library.counts.clone();
        let books: Vec<(usize, String, String, u32)> = library
            .book_tree()
            .into_iter()
            .map(|(depth, b)| (depth, b.id.clone(), b.name.clone(), b.count))
            .collect();
        let tags: Vec<(String, String, u32, kanso_core::TagColor)> = library
            .tags
            .iter()
            .map(|t| (t.id.clone(), t.name.clone(), t.count, t.color))
            .collect();

        let mut list = v_flex().pb_4();
        list = list
            .child(div().h_2())
            .child(self.row(
                "all",
                Filter::All,
                Some(self.icon(IconName::Inbox, cx)),
                "All Notes".into(),
                Some(counts.all),
                0,
                cx,
            ))
            .child(self.row(
                "pinned",
                Filter::Pinned,
                Some(self.icon(IconName::Star, cx)),
                "Pinned".into(),
                Some(counts.pinned),
                0,
                cx,
            ));

        let add_book = Button::new("add-book")
            .ghost()
            .xsmall()
            .icon(IconName::Plus)
            .tooltip("New notebook")
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewBook { parent: None })))
            .into_any_element();
        list = list.child(self.heading("NOTEBOOKS", Some(add_book), cx));
        for (depth, id, name, count) in books {
            let view = cx.entity();
            let book_id = id.clone();
            let row = self
                .row(
                    SharedString::from(format!("book-{id}")),
                    Filter::Book(id.clone()),
                    Some(self.icon(IconName::Folder, cx)),
                    name.into(),
                    Some(count),
                    depth,
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

        list = list.child(self.heading("STATUS", None, cx));
        for status in [
            NoteStatus::Active,
            NoteStatus::OnHold,
            NoteStatus::Completed,
            NoteStatus::Dropped,
        ] {
            let count = counts.by_status.get(&status).copied().unwrap_or(0);
            list = list.child(self.row(
                SharedString::from(format!("status-{}", status.as_str())),
                Filter::Status(status),
                Some(self.icon(status_icon(status), cx)),
                status.label().into(),
                Some(count),
                0,
                cx,
            ));
        }

        let add_tag = Button::new("add-tag")
            .ghost()
            .xsmall()
            .icon(IconName::Plus)
            .tooltip("New tag")
            .on_click(cx.listener(|_, _, _, cx| cx.emit(SidebarEvent::NewTag)))
            .into_any_element();
        list = list.child(self.heading("TAGS", Some(add_tag), cx));
        for (id, name, count, color) in tags {
            let view = cx.entity();
            let tag_id = id.clone();
            let dot = div()
                .size_2()
                .mx_1()
                .rounded_full()
                .bg(rgb_color(color.rgb()))
                .into_any_element();
            let row = self
                .row(
                    SharedString::from(format!("tag-{id}")),
                    Filter::Tag(id.clone()),
                    Some(dot),
                    name.into(),
                    Some(count),
                    0,
                    cx,
                )
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
                });
            list = list.child(row);
        }

        list = list.child(div().h_4());
        let trash_view = cx.entity();
        list = list.child(
            self.row(
                "trash",
                Filter::Trash,
                Some(self.icon(IconName::Delete, cx)),
                "Trash".into(),
                Some(counts.trash),
                0,
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

        div()
            .id("sidebar")
            .size_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(list)
    }
}

fn status_icon(status: NoteStatus) -> IconName {
    match status {
        NoteStatus::Active => IconName::Play,
        NoteStatus::OnHold => IconName::Pause,
        NoteStatus::Completed => IconName::CircleCheck,
        NoteStatus::Dropped => IconName::CircleX,
        NoteStatus::None => IconName::File,
    }
}
