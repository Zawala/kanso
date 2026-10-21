//! The icons a notebook can show: a fixed set to choose from, kept in the
//! notebook by name ([`kanso_core::model::BOOK_ICON_FIELD`]) so it syncs, and
//! the picker that chooses one.

use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::Book;
use kanso_core::model::BOOK_ICON_FIELD;

use crate::theme::Colors;

/// One icon a notebook can show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    /// What the notebook keeps: the Lucide name.
    pub name: &'static str,
    /// What the picker's tooltip says.
    pub label: &'static str,
    pub icon: Lucide,
}

const fn choice(name: &'static str, label: &'static str, icon: Lucide) -> Choice {
    Choice { name, label, icon }
}

/// The icon a notebook shows until another is chosen.
pub const DEFAULT: Lucide = Lucide::Notebook;

/// The icons to choose from, in the picker's order: notebooks and folders,
/// work, study and making things, life.
pub const CHOICES: &[Choice] = &[
    choice("notebook", "Notebook", Lucide::Notebook),
    choice("notebook-pen", "Journal", Lucide::NotebookPen),
    choice("book-open", "Book", Lucide::BookOpen),
    choice("folder", "Folder", Lucide::Folder),
    choice("archive", "Archive", Lucide::Archive),
    choice("inbox", "Inbox", Lucide::Inbox),
    choice("briefcase", "Work", Lucide::Briefcase),
    choice("building-2", "Company", Lucide::Building2),
    choice("users", "People", Lucide::Users),
    choice("user", "Person", Lucide::User),
    choice("bot", "Agents", Lucide::Bot),
    choice("code", "Code", Lucide::Code),
    choice("rocket", "Project", Lucide::Rocket),
    choice("lightbulb", "Ideas", Lucide::Lightbulb),
    choice("target", "Goals", Lucide::Target),
    choice("clipboard-list", "Tasks", Lucide::ClipboardList),
    choice("calendar", "Calendar", Lucide::Calendar),
    choice("graduation-cap", "Study", Lucide::GraduationCap),
    choice("flask-conical", "Research", Lucide::FlaskConical),
    choice("palette", "Design", Lucide::Palette),
    choice("pen-tool", "Drawing", Lucide::PenTool),
    choice("music", "Music", Lucide::Music),
    choice("camera", "Photos", Lucide::Camera),
    choice("film", "Film", Lucide::Film),
    choice("heart", "Favourites", Lucide::Heart),
    choice("star", "Starred", Lucide::Star),
    choice("sparkles", "Inspiration", Lucide::Sparkles),
    choice("house", "Home", Lucide::House),
    choice("plane", "Travel", Lucide::Plane),
    choice("car", "Car", Lucide::Car),
    choice("truck", "Logistics", Lucide::Truck),
    choice("globe", "World", Lucide::Globe),
    choice("wallet", "Money", Lucide::Wallet),
    choice("shopping-cart", "Shopping", Lucide::ShoppingCart),
    choice("utensils", "Food", Lucide::Utensils),
    choice("coffee", "Coffee", Lucide::Coffee),
    choice("dumbbell", "Fitness", Lucide::Dumbbell),
    choice("leaf", "Nature", Lucide::Leaf),
    choice("gamepad-2", "Games", Lucide::Gamepad2),
    choice("shield", "Private", Lucide::Shield),
];

/// The choice named `name`, if it is one.
pub fn find(name: &str) -> Option<&'static Choice> {
    CHOICES.iter().find(|choice| choice.name == name)
}

/// The name of the icon `book` keeps, if it is one of the choices.
pub fn chosen(book: &Book) -> Option<&'static str> {
    let name = book.extra.get(BOOK_ICON_FIELD)?.as_str()?;
    find(name).map(|choice| choice.name)
}

/// The icon `book` shows: its chosen one, else [`DEFAULT`].
pub fn book_icon(book: &Book) -> Lucide {
    chosen(book)
        .and_then(find)
        .map_or(DEFAULT, |choice| choice.icon)
}

/// Columns in the picker's grid.
const COLUMNS: usize = 8;
const CELL: f32 = 36.;
/// The picker's width: the grid and the dialog's padding.
pub const PICKER_WIDTH: f32 = COLUMNS as f32 * (CELL + 4.) + 44.;

type PickHandler = Rc<dyn Fn(Option<&'static str>, &mut Window, &mut App)>;

/// Opens the picker for notebook `name`, whose icon is `current` (by name).
/// `on_pick` gets the chosen icon's name, or `None` for the default, and
/// the picker closes.
pub fn open_picker(
    name: &str,
    current: Option<&'static str>,
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(Option<&'static str>, &mut Window, &mut App) + 'static,
) {
    let title = SharedString::from(format!("Icon for \u{201c}{name}\u{201d}"));
    let on_pick: PickHandler = Rc::new(on_pick);
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let cells = CHOICES.iter().enumerate().map(|(ix, choice)| {
            let selected = current == Some(choice.name)
                || (current.is_none() && choice.icon == DEFAULT && ix == 0);
            let on_pick = on_pick.clone();
            let name = choice.name;
            div()
                .id(("book-icon", ix))
                .debug_selector(move || format!("book-icon-{name}"))
                .size(px(CELL))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .cursor_pointer()
                .when(selected, |el| {
                    el.bg(colors.sidebar_active)
                        .border_1()
                        .border_color(colors.accent)
                })
                .when(!selected, |el| el.hover(|el| el.bg(colors.sidebar_hover)))
                .child(
                    Icon::new(choice.icon)
                        .size(px(18.))
                        .text_color(if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        }),
                )
                .tooltip(move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(choice_label(name)).build(window, cx)
                })
                .on_click(move |_, window, cx| {
                    window.close_dialog(cx);
                    // The first choice is the default: nothing to keep.
                    let picked = (name != CHOICES[0].name).then_some(name);
                    on_pick(picked, window, cx);
                })
        });
        let mut cells: Vec<_> = cells.collect();
        let mut rows: Vec<Div> = Vec::new();
        while !cells.is_empty() {
            let rest = cells.split_off(cells.len().min(COLUMNS));
            rows.push(
                h_flex()
                    .gap(px(4.))
                    .children(std::mem::replace(&mut cells, rest)),
            );
        }
        let reset = on_pick.clone();
        alert
            .title(title.clone())
            .width(px(PICKER_WIDTH))
            .child(v_flex().gap(px(4.)).children(rows))
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("book-icon-default")
                            .small()
                            .label("Use Default")
                            .disabled(current.is_none())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                reset(None, window, cx);
                            }),
                    )
                    .child(
                        Button::new("book-icon-cancel")
                            .small()
                            .primary()
                            .label("Done")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
    });
}

fn choice_label(name: &str) -> &'static str {
    find(name).map_or("Notebook", |choice| choice.label)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui_kit::AssetSource as _;
    use kanso_core::Book;
    use serde_json::json;

    use super::{CHOICES, DEFAULT, book_icon, chosen};
    use crate::assets::KansoAssets;

    fn book(icon: serde_json::Value) -> Book {
        serde_json::from_value(json!({"_id": "book:a", "name": "A", "kansoIcon": icon})).unwrap()
    }

    #[test]
    fn every_choice_is_bundled_and_named_once() {
        let mut names = HashSet::new();
        for choice in CHOICES {
            assert!(names.insert(choice.name), "{} twice", choice.name);
            let path = choice.icon.path();
            assert!(
                matches!(KansoAssets.load(&path), Ok(Some(_))),
                "{}: {path}",
                choice.name
            );
            // The name kept is the icon's own file name.
            assert!(path.ends_with(&format!("/{}.svg", choice.name)), "{path}");
        }
        assert_eq!(CHOICES[0].icon, DEFAULT);
    }

    #[test]
    fn a_notebook_shows_its_chosen_icon_or_the_default() {
        assert_eq!(
            book_icon(&book(json!("rocket"))),
            gpui_kit::assets::IconName::Rocket
        );
        assert_eq!(chosen(&book(json!("rocket"))), Some("rocket"));
        // Unknown names, other types and none all show the default.
        for value in [json!("not-an-icon"), json!(3), json!(null)] {
            assert_eq!(book_icon(&book(value.clone())), DEFAULT, "{value}");
            assert_eq!(chosen(&book(value)), None);
        }
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(book_icon(&plain), DEFAULT);
    }
}
