//! The icons a notebook can show: a set to choose from, kept in the notebook
//! by name ([`kanso_core::model::BOOK_ICON_FIELD`]) so it syncs, and the
//! picker that finds and chooses one. The icon theme in use draws them
//! all, and can add more of its own, kept as `<theme>/<icon>`.

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
use kanso_core::roles::{ROLE_FIELD, Role};

use crate::icon_theme_file::is_valid_name;
use crate::icon_themes::{self, IconTheme, Slot};
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

/// The first choice in the picker, which stands for the default: a
/// notebook keeps no icon for it.
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

/// What a notebook shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotebookIcon {
    /// One of the built-in choices, which the icon theme may draw.
    Choice(Lucide),
    /// The library's own: the default notebook, open or locked, or a
    /// notebook with a job, such as the Inbox.
    Slot(Slot),
    /// An icon the theme in use adds, by the path it is drawn under.
    Theme(SharedString),
}

impl NotebookIcon {
    /// The path it is drawn under (before the icon theme draws it).
    pub fn path(&self) -> SharedString {
        match self {
            NotebookIcon::Choice(icon) => gpui_kit::component::IconNamed::path(*icon),
            NotebookIcon::Slot(slot) => icon_themes::slot_path(*slot),
            NotebookIcon::Theme(path) => path.clone(),
        }
    }
}

impl From<NotebookIcon> for Icon {
    fn from(icon: NotebookIcon) -> Icon {
        Icon::default().path(icon.path())
    }
}

/// `<theme>/<icon>` as its two names, if it has that shape. Pure.
pub fn split_key(key: &str) -> Option<(&str, &str)> {
    let (theme, icon) = key.split_once('/')?;
    (is_valid_name(theme) && is_valid_name(icon)).then_some((theme, icon))
}

/// The icon `book` keeps, as kept: a built-in choice's name, or
/// `<theme>/<icon>` for an icon theme's, whether or not that theme is in use
/// (an icon pack's from before icon themes is kept the same way).
pub fn chosen(book: &Book) -> Option<String> {
    let name = book.extra.get(BOOK_ICON_FIELD)?.as_str()?;
    (find(name).is_some() || split_key(name).is_some()).then(|| name.to_owned())
}

/// The job `book` has, if any.
pub fn role(book: &Book) -> Option<Role> {
    Role::of_book(
        &book.id,
        book.extra.get(ROLE_FIELD).and_then(|value| value.as_str()),
    )
}

/// The icon a kept name shows under `theme`: its built-in choice, the
/// theme's own icon when it is the theme's, else the library's icon for a
/// notebook with `role`, or for any notebook. Pure.
pub fn icon_for(name: Option<&str>, role: Option<Role>, theme: &IconTheme) -> NotebookIcon {
    if let Some(name) = name {
        if let Some(choice) = find(name) {
            return NotebookIcon::Choice(choice.icon);
        }
        if let Some((id, icon)) = split_key(name)
            && id == theme.id
            && theme.extra(icon).is_some()
        {
            return NotebookIcon::Theme(icon_themes::extra_path(theme, icon));
        }
    }
    NotebookIcon::Slot(match role {
        Some(Role::Inbox) => Slot::Inbox,
        Some(Role::Journal) => Slot::Journal,
        Some(Role::Templates) => Slot::Templates,
        None => Slot::Notebook,
    })
}

/// The icon `book` keeps, drawn by the theme in use.
pub fn book_icon(book: &Book, cx: &App) -> NotebookIcon {
    icon_for(
        chosen(book).as_deref(),
        role(book),
        &icon_themes::active(cx),
    )
}

/// What a notebook shows for `icon`: the lock in its place while it is
/// `locked`, by its own lock or a notebook it is inside, so its own icon
/// says nothing about it then. Pure.
pub fn shown(icon: NotebookIcon, locked: bool) -> NotebookIcon {
    if locked {
        NotebookIcon::Slot(Slot::NotebookLocked)
    } else {
        icon
    }
}

/// What a notebook opened in the sidebar shows: the open notebook in place
/// of the plain one; any other icon as it is. Pure.
pub fn opened(icon: NotebookIcon, open: bool) -> NotebookIcon {
    match icon {
        NotebookIcon::Slot(Slot::Notebook) if open => NotebookIcon::Slot(Slot::NotebookOpen),
        icon => icon,
    }
}

/// The words of `query`, lowercased, split at spaces and dashes.
fn words(query: &str) -> Vec<String> {
    query
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether every word of `query` starts a word of `label`, `name` or
/// `keywords`, ignoring case. No words match everything. Pure.
pub fn matches(query: &[String], label: &str, name: &str, keywords: &str) -> bool {
    let text = format!("{label} {name} {keywords}");
    let theirs = words(&text);
    query
        .iter()
        .all(|word| theirs.iter().any(|their| their.starts_with(word.as_str())))
}

/// The built-in choices a search for `query` finds, in the picker's order:
/// see [`matches`]. Pure.
#[cfg(test)]
pub fn matching(query: &str) -> Vec<&'static Choice> {
    let query = words(query);
    CHOICES
        .iter()
        .filter(|choice| matches(&query, choice.label, choice.name, choice.keywords))
        .collect()
}

/// One icon in the picker.
#[derive(Clone)]
struct Entry {
    /// What the notebook keeps.
    key: SharedString,
    label: SharedString,
    keywords: SharedString,
    icon: NotebookIcon,
}

/// The picker's sections: Kanso's icons, then the icon theme's own, if it
/// adds any, under its name.
fn sections(theme: &IconTheme) -> Vec<(SharedString, Vec<Entry>)> {
    let builtin = CHOICES
        .iter()
        .map(|choice| Entry {
            key: choice.name.into(),
            label: choice.label.into(),
            keywords: choice.keywords.into(),
            icon: NotebookIcon::Choice(choice.icon),
        })
        .collect();
    let mut sections = vec![(SharedString::from("Kanso"), builtin)];
    if !theme.extras.is_empty() {
        let entries = theme
            .extras
            .iter()
            .map(|icon| Entry {
                key: format!("{}/{}", theme.id, icon.name).into(),
                label: icon.label.clone().into(),
                keywords: format!("{} {}", icon.keywords, theme.name).into(),
                icon: NotebookIcon::Theme(icon_themes::extra_path(theme, &icon.name)),
            })
            .collect();
        sections.push((theme.name.clone().into(), entries));
    }
    sections
}

/// Columns in the picker's grid, and the rows it shows before scrolling.
const COLUMNS: usize = 8;
/// Most icons a section draws; typing finds the rest.
const SECTION_LIMIT: usize = 240;
const VISIBLE_ROWS: usize = 6;
const CELL: f32 = 36.;
const GAP: f32 = 4.;
/// The picker's width: the grid and the dialog's padding.
pub const PICKER_WIDTH: f32 = COLUMNS as f32 * (CELL + GAP) + 44.;

type PickHandler = Rc<dyn Fn(Option<String>, &mut Window, &mut App)>;

/// The picker's body: a search field over the grid of icons it finds,
/// Kanso's and the icon theme's.
pub struct IconPicker {
    search: Entity<InputState>,
    current: Option<String>,
    sections: Vec<(SharedString, Vec<Entry>)>,
    /// What the search last found, for the query it found it for.
    found: Option<(String, Vec<Found>)>,
    on_pick: PickHandler,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

/// A section's icons the search found: the first [`SECTION_LIMIT`] of
/// them, and how many there are in all.
#[derive(Clone)]
struct Found {
    title: SharedString,
    entries: Vec<Entry>,
    total: usize,
}

impl IconPicker {
    fn new(
        current: Option<String>,
        sections: Vec<(SharedString, Vec<Entry>)>,
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
                    let first = this
                        .found(cx)
                        .into_iter()
                        .flat_map(|found| found.entries)
                        .next();
                    if let Some(first) = first {
                        this.pick(Some(first.key.to_string()), window, cx);
                    }
                }
                _ => {}
            },
        );
        IconPicker {
            search,
            current,
            sections,
            found: None,
            on_pick,
            scroll: ScrollHandle::new(),
            _subscription: subscription,
        }
    }

    /// The sections with the icons the search finds, leaving out the empty,
    /// worked out again only when the search changes.
    fn found(&mut self, cx: &App) -> Vec<Found> {
        let query = self.search.read(cx).value().to_string();
        if let Some((kept, found)) = &self.found
            && *kept == query
        {
            return found.clone();
        }
        let words = words(&query);
        let found: Vec<Found> = self
            .sections
            .iter()
            .filter_map(|(title, entries)| {
                let mut total = 0;
                let mut shown = Vec::new();
                for entry in entries {
                    if matches(&words, &entry.label, &entry.key, &entry.keywords) {
                        total += 1;
                        if shown.len() < SECTION_LIMIT {
                            shown.push(entry.clone());
                        }
                    }
                }
                (total > 0).then(|| Found {
                    title: title.clone(),
                    entries: shown,
                    total,
                })
            })
            .collect();
        self.found = Some((query, found.clone()));
        found
    }

    /// Closes the picker and hands on `key`; Kanso's first icon is the
    /// default, so nothing is kept for it.
    fn pick(&self, key: Option<String>, window: &mut Window, cx: &mut App) {
        let key = key.filter(|key| key != CHOICES[0].name);
        window.close_dialog(cx);
        (self.on_pick)(key, window, cx);
    }

    /// Types `query` into the search, as a person would (tests, snapshots).
    #[cfg(test)]
    pub fn search_for(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        let query = query.to_owned();
        self.search
            .update(cx, |search, cx| search.set_value(query, window, cx));
        cx.notify();
    }

    /// The keys of the icons showing, in order.
    #[cfg(test)]
    pub fn shown(&mut self, cx: &App) -> Vec<String> {
        self.found(cx)
            .into_iter()
            .flat_map(|found| found.entries)
            .map(|entry| entry.key.to_string())
            .collect()
    }
}

// no-prop-test: drawing; `matches` has the properties and the snapshots
// show the picker.
impl Render for IconPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let found = self.found(cx);
        let current: SharedString = self
            .current
            .clone()
            .unwrap_or_else(|| CHOICES[0].name.to_owned())
            .into();
        // With only Kanso's icons, no title is needed over them.
        let titled = self.sections.len() > 1;
        let mut blocks: Vec<AnyElement> = Vec::new();
        let mut ix: usize = 0;
        for Found {
            title,
            entries,
            total,
        } in &found
        {
            if titled {
                blocks.push(
                    div()
                        .pt(px(4.))
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.text_muted)
                        .child(title.clone())
                        .into_any_element(),
                );
            }
            let mut cells: Vec<_> = entries
                .iter()
                .map(|entry| {
                    ix += 1;
                    let selected = entry.key == current;
                    let key = entry.key.clone();
                    let selector = key.clone();
                    let label = entry.label.clone();
                    div()
                        .id(("book-icon", ix))
                        .debug_selector(move || format!("book-icon-{selector}"))
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
                        .child(Icon::from(entry.icon.clone()).size(px(18.)).text_color(
                            if selected {
                                colors.text
                            } else {
                                colors.text_secondary
                            },
                        ))
                        .tooltip(move |window, cx| {
                            gpui_kit::component::tooltip::Tooltip::new(label.clone())
                                .build(window, cx)
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.pick(Some(key.to_string()), window, cx)
                        }))
                })
                .collect();
            while !cells.is_empty() {
                let rest = cells.split_off(cells.len().min(COLUMNS));
                blocks.push(
                    h_flex()
                        .gap(px(GAP))
                        .children(std::mem::replace(&mut cells, rest))
                        .into_any_element(),
                );
            }
            if *total > entries.len() {
                blocks.push(
                    div()
                        .py(px(4.))
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(format!(
                            "{} of {} icons. Type to find the others.",
                            entries.len(),
                            total
                        ))
                        .into_any_element(),
                );
            }
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
                    .child(v_flex().gap(px(GAP)).children(blocks))
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

/// Opens the picker for notebook `name`, whose icon is `current` (as kept),
/// with the keyboard in its search. `on_pick` gets the chosen icon as the
/// notebook keeps it, or `None` for the default, and the picker closes.
/// Returns the picker.
pub fn open_picker(
    name: &str,
    current: Option<String>,
    window: &mut Window,
    cx: &mut App,
    on_pick: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
) -> Entity<IconPicker> {
    let title = SharedString::from(format!("Icon for \u{201c}{name}\u{201d}"));
    let on_pick: PickHandler = Rc::new(on_pick);
    let sections = sections(&icon_themes::active(cx));
    let has_icon = current.is_some();
    let picker = cx.new(|cx| IconPicker::new(current, sections, on_pick.clone(), window, cx));
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
                            .disabled(!has_icon)
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
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    use super::{CHOICES, DEFAULT, NotebookIcon, chosen, icon_for, matching, opened, role, shown};
    use crate::assets::KansoAssets;
    use crate::icon_themes::{self, Slot};

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
        let lucide = icon_themes::default_theme();
        let phosphor = &icon_themes::builtins()[1];
        let shows = |book: &Book| icon_for(chosen(book).as_deref(), role(book), lucide);
        let choice = NotebookIcon::Choice;
        let notebook = NotebookIcon::Slot(Slot::Notebook);
        assert_eq!(
            shows(&book(json!("rocket"))),
            choice(gpui_kit::assets::IconName::Rocket)
        );
        assert_eq!(chosen(&book(json!("rocket"))).as_deref(), Some("rocket"));
        // Unknown names, other types and none all show the default.
        for value in [json!("not-an-icon"), json!(3), json!(null), json!("../x/y")] {
            assert_eq!(shows(&book(value.clone())), notebook, "{value}");
            assert_eq!(chosen(&book(value)), None);
        }
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(shows(&plain), notebook);
        // Notebooks with a job have slots of their own.
        let inbox: Book =
            serde_json::from_value(json!({"_id": "book:inbox", "name": "Inbox"})).unwrap();
        assert_eq!(shows(&inbox), NotebookIcon::Slot(Slot::Inbox));
        let journal: Book = serde_json::from_value(
            json!({"_id": "book:x", "name": "Days", "kansoRole": "journal"}),
        )
        .unwrap();
        assert_eq!(shows(&journal), NotebookIcon::Slot(Slot::Journal));
        // Locked, any notebook shows the lock instead; open, the plain one
        // shows open and one with its own icon keeps it.
        let rocket = choice(gpui_kit::assets::IconName::Rocket);
        assert_eq!(shown(rocket.clone(), false), rocket);
        assert_eq!(
            shown(rocket.clone(), true),
            NotebookIcon::Slot(Slot::NotebookLocked)
        );
        assert_eq!(
            opened(notebook.clone(), true),
            NotebookIcon::Slot(Slot::NotebookOpen)
        );
        assert_eq!(opened(notebook.clone(), false), notebook);
        assert_eq!(opened(rocket.clone(), true), rocket);
        // A theme's icon is kept whether or not the theme is in use, shows
        // under its theme, and the default under any other; so does an old
        // icon pack's.
        let planet = book(json!("phosphor/planet"));
        assert_eq!(chosen(&planet).as_deref(), Some("phosphor/planet"));
        assert_eq!(shows(&planet), notebook);
        assert_eq!(
            icon_for(chosen(&planet).as_deref(), None, phosphor),
            NotebookIcon::Theme(icon_themes::extra_path(phosphor, "planet"))
        );
        let gone = book(json!("phosphor/not-there"));
        assert_eq!(icon_for(chosen(&gone).as_deref(), None, phosphor), notebook);
        let old_pack = book(json!("sketch/wave"));
        assert_eq!(chosen(&old_pack).as_deref(), Some("sketch/wave"));
        assert_eq!(icon_for(Some("sketch/wave"), None, phosphor), notebook);
        // Every icon is drawn by a path the icon theme can draw over.
        assert_eq!(rocket.path().as_ref(), "icons/rocket.svg");
        assert_eq!(notebook.path().as_ref(), "icons/library/notebook.svg");
    }

    /// The theme in use adds its own icons to the picker, in a section of
    /// their own, found by their words and the theme's name.
    #[test]
    fn a_themes_icons_are_found() {
        let phosphor = &icon_themes::builtins()[1];
        let sections = super::sections(phosphor);
        let titles: Vec<&str> = sections.iter().map(|(title, _)| title.as_ref()).collect();
        assert_eq!(titles, vec!["Kanso", "Phosphor"]);
        assert_eq!(super::sections(icon_themes::default_theme()).len(), 1);
        let found = |query: &str| -> Vec<String> {
            let query = super::words(query);
            sections
                .iter()
                .flat_map(|(_, entries)| entries)
                .filter(|e| super::matches(&query, &e.label, &e.key, &e.keywords))
                .map(|e| e.key.to_string())
                .collect()
        };
        assert_eq!(found("orbit"), vec!["phosphor/planet".to_owned()]);
        assert!(found("phosphor").contains(&"phosphor/planet".to_owned()));
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

        let picked: Rc<RefCell<Vec<Option<String>>>> = Rc::default();
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
        assert_eq!(
            picker.update(cx, |picker, cx| picker.shown(cx)).len(),
            CHOICES.len()
        );
        window
            .update(cx, |_, window, cx| {
                picker.update(cx, |picker, cx| picker.search_for("piggy", window, cx))
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(
            picker.update(cx, |picker, cx| picker.shown(cx)),
            vec!["piggy-bank".to_owned()]
        );
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
        assert_eq!(*picked.borrow(), vec![Some("piggy-bank".to_owned())]);
        let open = window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap();
        assert!(!open, "the picker closed");
    }

    /// In a window, under Phosphor: a notebook keeping one of its icons
    /// shows it in the sidebar and the palette's notebooks; the Inbox shows
    /// its slot; a locked notebook shows the lock in both. Back on Kanso's
    /// own icons, the theme's icon gives way to the default.
    #[gpui_kit::test]
    fn a_themes_icon_shows_in_the_sidebar_and_palette(cx: &mut gpui_kit::TestAppContext) {
        use std::sync::Arc;

        use gpui_kit::AppContext as _;
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};

        use crate::palette::Mode;

        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        for book in [
            json!({"name": "Trips", "kansoIcon": "phosphor/planet"}),
            json!({"name": "Vault", "kansoIcon": "phosphor/planet", "kansoLock": {"v": 1}}),
            json!({"_id": "book:inbox", "name": "Inbox", "kansoRole": "inbox"}),
        ] {
            store
                .put(DocType::Book, book, Put::Upsert, Origin::Ui)
                .unwrap();
        }
        let data_dir = dir.path().to_path_buf();
        let mut config = crate::config::AppConfig::default();
        config.appearance.icon_theme = Some("phosphor".into());
        let (window, workspace) = cx.update(|cx| {
            gpui_kit::init(cx);
            cx.set_global(crate::workspace::AppState {
                store,
                data_dir,
                config,
            });
            cx.set_global(crate::services::Services::default());
            crate::workspace::bind_keys(cx);
            crate::icon_themes::init(cx);
            gpui_kit::open_window(Default::default(), cx, |window, cx| {
                cx.new(|cx| crate::workspace::Workspace::new(window, cx))
            })
            .unwrap()
        });
        cx.run_until_parked();

        let phosphor = icon_themes::builtins()[1].clone();
        let planet = NotebookIcon::Theme(icon_themes::extra_path(&phosphor, "planet"));
        let lock = NotebookIcon::Slot(Slot::NotebookLocked);
        let inbox = NotebookIcon::Slot(Slot::Inbox);
        let rows = |cx: &mut gpui_kit::TestAppContext| {
            cx.read(|cx| {
                workspace
                    .read(cx)
                    .sidebar
                    .read(cx)
                    .book_rows_for_test(cx)
                    .into_iter()
                    .map(|(name, icon, _)| (name, icon))
                    .collect::<Vec<_>>()
            })
        };
        assert_eq!(
            rows(cx),
            vec![
                ("Inbox".to_owned(), inbox.clone()),
                ("Trips".to_owned(), planet.clone()),
                ("Vault".to_owned(), lock.clone())
            ]
        );
        let palette = cx.read(|cx| workspace.read(cx).palette.clone());
        window
            .update(cx, |_, window, cx| {
                palette.update(cx, |palette, cx| {
                    palette.open(
                        Mode::Pick(crate::commands::Picker::GoToNotebook),
                        window,
                        cx,
                    )
                })
            })
            .unwrap();
        cx.run_until_parked();
        let listed = cx.read(|cx| palette.read(cx).item_icons());
        assert_eq!(
            listed,
            vec![
                ("Inbox".to_owned(), inbox.path()),
                ("Trips".to_owned(), planet.path()),
                ("Vault".to_owned(), lock.path())
            ]
        );
        cx.update(|cx| icon_themes::choose(None, cx));
        cx.run_until_parked();
        assert_eq!(
            rows(cx)[1],
            ("Trips".to_owned(), NotebookIcon::Slot(Slot::Notebook))
        );
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

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
