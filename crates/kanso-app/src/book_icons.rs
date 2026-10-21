//! The icons a notebook can show: a set to choose from, kept in the notebook
//! by name ([`kanso_core::model::BOOK_ICON_FIELD`]) so it syncs, and the
//! picker that finds and chooses one.

use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
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
    /// More words the search finds it by, spaced.
    pub keywords: &'static str,
    pub icon: Lucide,
}

const fn choice(
    name: &'static str,
    label: &'static str,
    keywords: &'static str,
    icon: Lucide,
) -> Choice {
    Choice {
        name,
        label,
        keywords,
        icon,
    }
}

/// The icon a notebook shows until another is chosen.
pub const DEFAULT: Lucide = Lucide::Notebook;

/// The icons to choose from, in the picker's order: notebooks and files,
/// work and code, plans, study and making things, then life.
pub const CHOICES: &[Choice] = &[
    choice(
        "notebook",
        "Notebook",
        "book journal default",
        Lucide::Notebook,
    ),
    choice(
        "notebook-pen",
        "Journal",
        "diary writing log",
        Lucide::NotebookPen,
    ),
    choice(
        "notebook-text",
        "Notes",
        "text writing",
        Lucide::NotebookText,
    ),
    choice("book-open", "Book", "reading library", Lucide::BookOpen),
    choice(
        "book-marked",
        "Bookmarked",
        "reading saved",
        Lucide::BookMarked,
    ),
    choice("library", "Library", "books shelf reading", Lucide::Library),
    choice("folder", "Folder", "files directory", Lucide::Folder),
    choice(
        "folder-open",
        "Open folder",
        "files directory",
        Lucide::FolderOpen,
    ),
    choice(
        "folder-heart",
        "Favourite folder",
        "files love",
        Lucide::FolderHeart,
    ),
    choice(
        "folder-lock",
        "Locked folder",
        "private secret files",
        Lucide::FolderLock,
    ),
    choice("archive", "Archive", "old storage box", Lucide::Archive),
    choice("inbox", "Inbox", "mail incoming", Lucide::Inbox),
    choice("mail", "Mail", "email letter", Lucide::Mail),
    choice("file-text", "Document", "page doc paper", Lucide::FileText),
    choice("files", "Files", "documents pages", Lucide::Files),
    choice(
        "scroll-text",
        "Scroll",
        "history story text",
        Lucide::ScrollText,
    ),
    choice(
        "sticky-note",
        "Sticky note",
        "memo post-it",
        Lucide::StickyNote,
    ),
    choice("bookmark", "Bookmark", "saved later", Lucide::Bookmark),
    choice(
        "briefcase",
        "Work",
        "job office business",
        Lucide::Briefcase,
    ),
    choice(
        "building-2",
        "Company",
        "office business org",
        Lucide::Building2,
    ),
    choice(
        "landmark",
        "Institution",
        "bank government",
        Lucide::Landmark,
    ),
    choice("users", "People", "team group", Lucide::Users),
    choice("user", "Person", "me profile", Lucide::User),
    choice("contact", "Contacts", "people address", Lucide::Contact),
    choice(
        "handshake",
        "Deals",
        "partners clients agreement",
        Lucide::Handshake,
    ),
    choice("bot", "Agents", "ai robot assistant", Lucide::Bot),
    choice("brain", "Brain", "mind thinking ai", Lucide::Brain),
    choice("cpu", "Hardware", "chip computer", Lucide::Cpu),
    choice("code", "Code", "programming dev", Lucide::Code),
    choice(
        "terminal",
        "Terminal",
        "shell command cli",
        Lucide::Terminal,
    ),
    choice(
        "git-branch",
        "Git",
        "branch version source",
        Lucide::GitBranch,
    ),
    choice("database", "Database", "data sql storage", Lucide::Database),
    choice("server", "Server", "backend hosting", Lucide::Server),
    choice("cloud", "Cloud", "hosting sky", Lucide::Cloud),
    choice("bug", "Bugs", "issues debug", Lucide::Bug),
    choice("wrench", "Tools", "fix settings repair", Lucide::Wrench),
    choice("hammer", "Build", "construct make", Lucide::Hammer),
    choice("rocket", "Project", "launch startup", Lucide::Rocket),
    choice("lightbulb", "Ideas", "idea thought", Lucide::Lightbulb),
    choice("target", "Goals", "aim objective", Lucide::Target),
    choice("flag", "Milestones", "goal marker", Lucide::Flag),
    choice("trophy", "Wins", "award achievement", Lucide::Trophy),
    choice(
        "chart-line",
        "Metrics",
        "stats analytics graph",
        Lucide::ChartLine,
    ),
    choice("chart-pie", "Reports", "stats analytics", Lucide::ChartPie),
    choice(
        "presentation",
        "Presentations",
        "slides talk",
        Lucide::Presentation,
    ),
    choice(
        "clipboard-list",
        "Tasks",
        "todo checklist",
        Lucide::ClipboardList,
    ),
    choice("list-checks", "Checklists", "todo done", Lucide::ListChecks),
    choice("calendar", "Calendar", "date schedule", Lucide::Calendar),
    choice(
        "calendar-days",
        "Plans",
        "schedule week",
        Lucide::CalendarDays,
    ),
    choice("clock", "Time", "hours schedule", Lucide::Clock),
    choice("alarm-clock", "Reminders", "alarm wake", Lucide::AlarmClock),
    choice("hourglass", "Waiting", "later time", Lucide::Hourglass),
    choice(
        "graduation-cap",
        "Study",
        "school learning education",
        Lucide::GraduationCap,
    ),
    choice("school", "School", "education class", Lucide::School),
    choice(
        "flask-conical",
        "Research",
        "science lab experiment",
        Lucide::FlaskConical,
    ),
    choice("microscope", "Science", "research lab", Lucide::Microscope),
    choice("atom", "Physics", "science atom", Lucide::Atom),
    choice(
        "calculator",
        "Maths",
        "numbers calculate",
        Lucide::Calculator,
    ),
    choice(
        "languages",
        "Languages",
        "translate words",
        Lucide::Languages,
    ),
    choice("palette", "Design", "art colour", Lucide::Palette),
    choice("pen-tool", "Drawing", "vector design", Lucide::PenTool),
    choice("brush", "Painting", "art paint", Lucide::Brush),
    choice("pencil", "Writing", "draft edit", Lucide::Pencil),
    choice("feather", "Poetry", "writing quill", Lucide::Feather),
    choice("image", "Images", "pictures photos", Lucide::Image),
    choice("camera", "Photos", "photography pictures", Lucide::Camera),
    choice("film", "Film", "movies video", Lucide::Film),
    choice(
        "clapperboard",
        "Production",
        "video movie",
        Lucide::Clapperboard,
    ),
    choice("mic", "Podcast", "audio recording voice", Lucide::Mic),
    choice("music", "Music", "songs audio", Lucide::Music),
    choice("headphones", "Listening", "audio music", Lucide::Headphones),
    choice("guitar", "Guitar", "music instrument", Lucide::Guitar),
    choice("gamepad-2", "Games", "gaming play", Lucide::Gamepad2),
    choice("dices", "Board games", "dice games", Lucide::Dices),
    choice("puzzle", "Puzzles", "games pieces", Lucide::Puzzle),
    choice("heart", "Favourites", "love liked", Lucide::Heart),
    choice("star", "Starred", "favourite important", Lucide::Star),
    choice("sparkles", "Inspiration", "magic ideas", Lucide::Sparkles),
    choice("gift", "Gifts", "presents birthday", Lucide::Gift),
    choice(
        "party-popper",
        "Celebrations",
        "party events",
        Lucide::PartyPopper,
    ),
    choice("house", "Home", "house family", Lucide::House),
    choice("bed", "Sleep", "rest bedroom", Lucide::Bed),
    choice("sofa", "Living", "home furniture", Lucide::Sofa),
    choice("key", "Keys", "access passwords", Lucide::Key),
    choice("lock", "Locked", "private secret", Lucide::Lock),
    choice("shield", "Private", "security safe", Lucide::Shield),
    choice("plane", "Travel", "trips flight", Lucide::Plane),
    choice("map", "Maps", "places directions", Lucide::Map),
    choice("map-pin", "Places", "location address", Lucide::MapPin),
    choice(
        "compass",
        "Explore",
        "navigation direction",
        Lucide::Compass,
    ),
    choice("mountain", "Outdoors", "hiking nature", Lucide::Mountain),
    choice("tent", "Camping", "outdoors trip", Lucide::Tent),
    choice("car", "Car", "driving vehicle", Lucide::Car),
    choice("bike", "Cycling", "bicycle ride", Lucide::Bike),
    choice(
        "truck",
        "Logistics",
        "delivery shipping fleet",
        Lucide::Truck,
    ),
    choice("bus", "Transport", "commute bus", Lucide::Bus),
    choice("train-front", "Trains", "rail commute", Lucide::TrainFront),
    choice("ship", "Shipping", "boat sea", Lucide::Ship),
    choice("globe", "World", "international web", Lucide::Globe),
    choice("wallet", "Money", "finance budget", Lucide::Wallet),
    choice("piggy-bank", "Savings", "money budget", Lucide::PiggyBank),
    choice("coins", "Coins", "money cash", Lucide::Coins),
    choice("credit-card", "Payments", "card money", Lucide::CreditCard),
    choice("receipt", "Receipts", "expenses bills", Lucide::Receipt),
    choice("banknote", "Cash", "money bills", Lucide::Banknote),
    choice(
        "shopping-cart",
        "Shopping",
        "groceries buy",
        Lucide::ShoppingCart,
    ),
    choice(
        "shopping-bag",
        "Purchases",
        "shopping buy",
        Lucide::ShoppingBag,
    ),
    choice("store", "Store", "shop retail", Lucide::Store),
    choice("tag", "Labels", "tag price", Lucide::Tag),
    choice(
        "utensils",
        "Food",
        "recipes meals cooking",
        Lucide::Utensils,
    ),
    choice("chef-hat", "Cooking", "recipes kitchen", Lucide::ChefHat),
    choice("coffee", "Coffee", "cafe drink", Lucide::Coffee),
    choice("cake", "Baking", "dessert birthday", Lucide::Cake),
    choice("apple", "Health food", "fruit nutrition", Lucide::Apple),
    choice("dumbbell", "Fitness", "gym workout", Lucide::Dumbbell),
    choice(
        "heart-pulse",
        "Health",
        "medical wellbeing",
        Lucide::HeartPulse,
    ),
    choice(
        "stethoscope",
        "Medical",
        "doctor health",
        Lucide::Stethoscope,
    ),
    choice("pill", "Medicine", "health pharmacy", Lucide::Pill),
    choice("baby", "Baby", "kids family", Lucide::Baby),
    choice("dog", "Pets", "dog animals", Lucide::Dog),
    choice("cat", "Cat", "pets animals", Lucide::Cat),
    choice("paw-print", "Animals", "pets", Lucide::PawPrint),
    choice("leaf", "Nature", "plants green", Lucide::Leaf),
    choice("flower", "Garden", "flowers plants", Lucide::Flower),
    choice("trees", "Forest", "trees nature", Lucide::Trees),
    choice("sun", "Summer", "sunny weather", Lucide::Sun),
    choice("moon", "Night", "dreams evening", Lucide::Moon),
    choice("cloud-rain", "Weather", "rain forecast", Lucide::CloudRain),
    choice("snowflake", "Winter", "snow cold", Lucide::Snowflake),
    choice("flame", "Hot", "fire trending", Lucide::Flame),
    choice("zap", "Energy", "power fast", Lucide::Zap),
    choice("droplet", "Water", "drop liquid", Lucide::Droplet),
    choice("church", "Faith", "church religion", Lucide::Church),
    choice(
        "hand-heart",
        "Charity",
        "giving care volunteer",
        Lucide::HandHeart,
    ),
    choice("newspaper", "News", "articles press", Lucide::Newspaper),
    choice("rss", "Feeds", "blogs subscriptions", Lucide::Rss),
    choice("link", "Links", "bookmarks web", Lucide::Link),
    choice(
        "message-circle",
        "Chats",
        "messages talk",
        Lucide::MessageCircle,
    ),
    choice("phone", "Calls", "phone contacts", Lucide::Phone),
    choice("video", "Meetings", "video calls", Lucide::Video),
    choice(
        "megaphone",
        "Marketing",
        "announcements promo",
        Lucide::Megaphone,
    ),
    choice(
        "lock-keyhole",
        "Vault",
        "secure private",
        Lucide::LockKeyhole,
    ),
    choice("eye", "Watching", "watchlist view", Lucide::Eye),
    choice("infinity", "Ongoing", "forever loop", Lucide::Infinity),
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

/// The choices a search for `query` finds, in the picker's order: those
/// whose label, name or keywords hold every word typed, at the start of one
/// of theirs, ignoring case. Nothing typed finds them all. Pure.
pub fn matching(query: &str) -> Vec<&'static Choice> {
    let words: Vec<String> = query
        .split(|c: char| c.is_whitespace() || c == '-')
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    CHOICES
        .iter()
        .filter(|choice| {
            let text =
                format!("{} {} {}", choice.label, choice.name, choice.keywords).to_lowercase();
            let theirs: Vec<&str> = text
                .split(|c: char| c.is_whitespace() || c == '-')
                .filter(|word| !word.is_empty())
                .collect();
            words
                .iter()
                .all(|word| theirs.iter().any(|their| their.starts_with(word.as_str())))
        })
        .collect()
}

/// Columns in the picker's grid, and the rows it shows before scrolling.
const COLUMNS: usize = 8;
const VISIBLE_ROWS: usize = 6;
const CELL: f32 = 36.;
const GAP: f32 = 4.;
/// The picker's width: the grid and the dialog's padding.
pub const PICKER_WIDTH: f32 = COLUMNS as f32 * (CELL + GAP) + 44.;

type PickHandler = Rc<dyn Fn(Option<&'static str>, &mut Window, &mut App)>;

/// The picker's body: a search field over the grid of icons it finds.
pub struct IconPicker {
    search: Entity<InputState>,
    current: Option<&'static str>,
    on_pick: PickHandler,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

impl IconPicker {
    fn new(
        current: Option<&'static str>,
        on_pick: PickHandler,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icons"));
        let subscription = cx.subscribe_in(
            &search,
            window,
            |this: &mut IconPicker, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => {
                    this.scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }
                // Enter picks the first icon found.
                InputEvent::PressEnter { .. } => {
                    let query = this.search.read(cx).value();
                    if let Some(first) = matching(&query).first() {
                        this.pick(Some(first.name), window, cx);
                    }
                }
                _ => {}
            },
        );
        IconPicker {
            search,
            current,
            on_pick,
            scroll: ScrollHandle::new(),
            _subscription: subscription,
        }
    }

    /// Closes the picker and hands on `name`; the first choice is the
    /// default, so nothing is kept for it.
    fn pick(&self, name: Option<&'static str>, window: &mut Window, cx: &mut App) {
        let name = name.filter(|name| *name != CHOICES[0].name);
        window.close_dialog(cx);
        (self.on_pick)(name, window, cx);
    }

    /// Types `query` into the search, as a person would (tests, snapshots).
    #[cfg(test)]
    pub fn search_for(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        let query = query.to_owned();
        self.search
            .update(cx, |search, cx| search.set_value(query, window, cx));
        cx.notify();
    }

    /// The names of the icons showing, in order.
    #[cfg(test)]
    pub fn shown(&self, cx: &App) -> Vec<&'static str> {
        matching(&self.search.read(cx).value())
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }
}

// no-prop-test: drawing; `matching` has the properties and the snapshot
// shows the picker.
impl Render for IconPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let found = matching(&self.search.read(cx).value());
        let current = self.current.unwrap_or(CHOICES[0].name);
        let cells: Vec<_> = found
            .iter()
            .enumerate()
            .map(|(ix, choice)| {
                let selected = choice.name == current;
                let name = choice.name;
                let label = choice.label;
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
                        gpui_kit::component::tooltip::Tooltip::new(label).build(window, cx)
                    })
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.pick(Some(name), window, cx)),
                    )
            })
            .collect();
        let mut cells = cells;
        let mut rows: Vec<Div> = Vec::new();
        while !cells.is_empty() {
            let rest = cells.split_off(cells.len().min(COLUMNS));
            rows.push(
                h_flex()
                    .gap(px(GAP))
                    .children(std::mem::replace(&mut cells, rest)),
            );
        }
        let height = VISIBLE_ROWS as f32 * (CELL + GAP);
        v_flex()
            .gap(px(10.))
            .child(Input::new(&self.search).small())
            .child(
                div()
                    .id("book-icon-grid")
                    .h(px(height))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(v_flex().gap(px(GAP)).children(rows))
                    .when(found.is_empty(), |el| {
                        el.child(
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(13.))
                                .text_color(colors.text_muted)
                                .child("No icons match. Try another word."),
                        )
                    }),
            )
    }
}

/// Opens the picker for notebook `name`, whose icon is `current` (by name),
/// with the keyboard in its search. `on_pick` gets the chosen icon's name,
/// or `None` for the default, and the picker closes. Returns the picker.
pub fn open_picker(
    name: &str,
    current: Option<&'static str>,
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(Option<&'static str>, &mut Window, &mut App) + 'static,
) -> Entity<IconPicker> {
    let title = SharedString::from(format!("Icon for \u{201c}{name}\u{201d}"));
    let on_pick: PickHandler = Rc::new(on_pick);
    let picker = cx.new(|cx| IconPicker::new(current, on_pick.clone(), window, cx));
    let search = picker.read(cx).search.clone();
    let first_build = std::cell::Cell::new(true);
    let view = picker.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            search.update(cx, |search, cx| search.focus(window, cx));
        }
        let reset = on_pick.clone();
        alert
            .title(title.clone())
            .width(px(PICKER_WIDTH))
            .child(view.clone())
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
    picker
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui_kit::AssetSource as _;
    use kanso_core::Book;
    use proptest::prelude::*;
    use serde_json::json;

    use super::{CHOICES, DEFAULT, book_icon, chosen, matching};
    use crate::assets::KansoAssets;

    fn book(icon: serde_json::Value) -> Book {
        serde_json::from_value(json!({"_id": "book:a", "name": "A", "kansoIcon": icon})).unwrap()
    }

    fn names(query: &str) -> Vec<&'static str> {
        matching(query)
            .into_iter()
            .map(|choice| choice.name)
            .collect()
    }

    #[test]
    fn every_choice_is_bundled_and_named_once() {
        let mut names = HashSet::new();
        let mut labels = HashSet::new();
        for choice in CHOICES {
            assert!(names.insert(choice.name), "{} twice", choice.name);
            assert!(labels.insert(choice.label), "label {} twice", choice.label);
            let path = gpui_kit::component::IconNamed::path(choice.icon);
            assert!(
                matches!(KansoAssets.load(&path), Ok(Some(_))),
                "{}: {path}",
                choice.name
            );
            // The name kept is the icon's own file name.
            assert!(path.ends_with(&format!("/{}.svg", choice.name)), "{path}");
        }
        assert!(CHOICES.len() >= 120, "{}", CHOICES.len());
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

    #[test]
    fn search_finds_icons_by_label_name_and_keyword() {
        assert_eq!(matching("").len(), CHOICES.len());
        // A keyword: several icons say money.
        let money = names("money");
        for expected in ["wallet", "piggy-bank", "coins", "banknote"] {
            assert!(money.contains(&expected), "{money:?}");
        }
        // The start of a word, any case, every word typed.
        assert!(names("ROCK").contains(&"rocket"));
        assert_eq!(names("pets cat"), vec!["cat"]);
        // A Lucide name with its dash.
        assert!(names("piggy-bank").contains(&"piggy-bank"));
        assert!(names("zzzz").is_empty());
        // Words match at their starts, not in their middles.
        assert!(!names("ocket").contains(&"rocket"));
    }

    struct Blank;

    impl gpui_kit::Render for Blank {
        fn render(
            &mut self,
            _: &mut gpui_kit::Window,
            _: &mut gpui_kit::Context<Self>,
        ) -> impl gpui_kit::IntoElement {
            gpui_kit::div()
        }
    }

    /// In a window: typing narrows the grid, Enter picks the first icon
    /// found and closes the picker.
    #[gpui_kit::test]
    fn typing_narrows_the_grid_and_enter_picks_the_first(cx: &mut gpui_kit::TestAppContext) {
        use std::cell::RefCell;
        use std::rc::Rc;

        use gpui_kit::AppContext as _;
        use gpui_kit::component::WindowExt as _;

        let picked: Rc<RefCell<Vec<Option<&'static str>>>> = Rc::default();
        let (window, _) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(Default::default(), cx, |_, cx| cx.new(|_| Blank)).unwrap()
        });
        cx.run_until_parked();
        let kept = picked.clone();
        let picker = window
            .update(cx, |_, window, cx| {
                super::open_picker("Finance", None, window, cx, move |name, _, _| {
                    kept.borrow_mut().push(name)
                })
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(cx.read(|cx| picker.read(cx).shown(cx)).len(), CHOICES.len());
        window
            .update(cx, |_, window, cx| {
                picker.update(cx, |picker, cx| picker.search_for("piggy", window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(cx.read(|cx| picker.read(cx).shown(cx)), vec!["piggy-bank"]);
        let search = cx.read(|cx| picker.read(cx).search.clone());
        window
            .update(cx, |_, window, cx| {
                search.update(cx, |_, cx| {
                    cx.emit(gpui_kit::component::input::InputEvent::PressEnter {
                        secondary: false,
                        shift: false,
                    })
                });
                let _ = window;
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(*picked.borrow(), vec![Some("piggy-bank")]);
        let open = window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap();
        assert!(!open, "the picker closed");
    }

    proptest! {
        /// Invariants: whatever is typed, the icons found keep the picker's
        /// order, and typing more words never finds more.
        #[test]
        fn prop_more_words_find_fewer_icons(first in "[a-z]{0,4}", second in "[a-z]{0,4}") {
            let one = names(&first);
            let both = names(&format!("{first} {second}"));
            prop_assert!(both.iter().all(|name| one.contains(name)));
            let order: Vec<usize> = one
                .iter()
                .map(|name| CHOICES.iter().position(|c| c.name == *name).unwrap())
                .collect();
            prop_assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
}
