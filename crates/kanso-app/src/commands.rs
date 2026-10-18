//! What the command palette can do, apart from the places and notes that
//! come from the library (notebooks, statuses, notes). Most commands
//! dispatch the same action as their menu item or shortcut, so the palette
//! shows the shortcut the keymap really has; a few open a second list in
//! the palette to pick a notebook, tag or status from.

use gpui_kit::Action;
use gpui_kit::assets::IconName;

use crate::settings::{
    OpenApiSettings, OpenAppearanceSettings, OpenEditorSettings, OpenGeneralSettings,
    OpenSyncSettings,
};
use crate::workspace::{
    ExportLibrary, ExportNote, FocusSearch, ImportInkdrop, NewNote, NewNotebook, NextNote,
    OpenNotePicker, OpenSettings, PreviousNote, Quit, SaveNote, SearchAllNotes, ShowEditor,
    ShowHistory, ShowPreview, ShowSplit, SyncNow, ToggleFocusMode, TogglePinned, ToggleSidebar,
    TrashNote,
};

/// The palette's sections, in the order they appear with nothing typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Note,
    View,
    GoTo,
    App,
}

impl Group {
    pub fn label(self) -> &'static str {
        match self {
            Group::Note => "Note actions",
            Group::View => "View",
            Group::GoTo => "Go to",
            Group::App => "App",
        }
    }
}

/// A second list the palette shows to choose from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Picker {
    /// The notebook to move the open note into.
    Notebook,
    Tag,
    Status,
    /// A notebook for the note list to show.
    GoToNotebook,
}

/// What choosing a command does.
#[derive(Clone, Copy, Debug)]
pub enum Run {
    /// Dispatches an action, as its menu item or shortcut would.
    Action(fn() -> Box<dyn Action>),
    /// Opens a list in the palette to pick from.
    Pick(Picker),
}

/// When a command applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Nothing,
    /// A note is open.
    Note,
    /// A note is open and isn't in the trash.
    LiveNote,
}

pub struct Command {
    /// Stable and unique, for tests and for remembering the selection.
    pub id: &'static str,
    pub label: &'static str,
    /// The label when the open note already has what the command toggles
    /// (for pinning: "Unpin note").
    pub label_when_pinned: Option<(&'static str, IconName)>,
    /// One or two sentences for the detail pane.
    pub description: &'static str,
    pub icon: IconName,
    /// Other words people might type for it.
    pub keywords: &'static [&'static str],
    pub group: Group,
    pub run: Run,
    pub needs: Needs,
}

impl Command {
    /// The action the command dispatches, if it dispatches one.
    pub fn action(&self) -> Option<Box<dyn Action>> {
        match self.run {
            Run::Action(action) => Some(action()),
            Run::Pick(_) => None,
        }
    }
}

macro_rules! action {
    ($action:expr) => {
        Run::Action(|| Box::new($action))
    };
}

static COMMANDS: &[Command] = &[
    // Note actions.
    Command {
        id: "new-note",
        label: "New note",
        label_when_pinned: None,
        description: "Creates a note in the current notebook and puts the cursor in its title.",
        icon: IconName::FilePlus,
        keywords: &["create", "add", "write"],
        group: Group::Note,
        run: action!(NewNote),
        needs: Needs::Nothing,
    },
    Command {
        id: "new-notebook",
        label: "New notebook…",
        label_when_pinned: None,
        description: "Creates a notebook. Notes can be moved into it afterwards.",
        icon: IconName::Notebook,
        keywords: &["create", "add", "folder"],
        group: Group::Note,
        run: action!(NewNotebook),
        needs: Needs::Nothing,
    },
    Command {
        id: "search",
        label: "Search all notes",
        label_when_pinned: None,
        description: "Shows all notes and moves to the search box above them, whatever notebook, tag or status was showing. It searches titles and text; book:, tag: and status: narrow it down.",
        icon: IconName::Search,
        keywords: &["find", "global", "everywhere", "query"],
        group: Group::Note,
        run: action!(SearchAllNotes),
        needs: Needs::Nothing,
    },
    Command {
        id: "search-list",
        label: "Search this list",
        label_when_pinned: None,
        description: "Moves to the search box above the note list, to search only the notes it shows: a notebook, a tag, a status or the trash.",
        icon: IconName::Search,
        keywords: &["find", "filter", "notebook", "query"],
        group: Group::Note,
        run: action!(FocusSearch),
        needs: Needs::Nothing,
    },
    Command {
        id: "open-note",
        label: "Open note…",
        label_when_pinned: None,
        description: "Finds a note by its title and opens it.",
        icon: IconName::FileText,
        keywords: &["go", "jump", "switch", "find", "title"],
        group: Group::Note,
        run: action!(OpenNotePicker),
        needs: Needs::Nothing,
    },
    Command {
        id: "pin",
        label: "Pin note",
        label_when_pinned: Some(("Unpin note", IconName::PinOff)),
        description: "Keeps the note at the top of every list, and under Pinned in the sidebar.",
        icon: IconName::Pin,
        keywords: &["pin", "unpin", "star", "favorite", "top"],
        group: Group::Note,
        run: action!(TogglePinned),
        needs: Needs::LiveNote,
    },
    Command {
        id: "move",
        label: "Move to notebook…",
        label_when_pinned: None,
        description: "Moves the note into another notebook. A note in the trash comes back out.",
        icon: IconName::NotebookText,
        keywords: &["notebook", "folder", "file", "restore"],
        group: Group::Note,
        run: Run::Pick(Picker::Notebook),
        needs: Needs::Note,
    },
    Command {
        id: "add-tag",
        label: "Add tag…",
        label_when_pinned: None,
        description: "Adds a tag to the note, or takes one off. Typing a new name creates the tag.",
        icon: IconName::Tag,
        keywords: &["tags", "label", "hashtag", "remove tag"],
        group: Group::Note,
        run: Run::Pick(Picker::Tag),
        needs: Needs::LiveNote,
    },
    Command {
        id: "status",
        label: "Set status…",
        label_when_pinned: None,
        description: "Marks the note Active, On hold, Completed or Dropped, or clears its status.",
        icon: IconName::CircleDot,
        keywords: &["active", "on hold", "completed", "dropped", "done"],
        group: Group::Note,
        run: Run::Pick(Picker::Status),
        needs: Needs::LiveNote,
    },
    Command {
        id: "history",
        label: "Show history…",
        label_when_pinned: None,
        description: "Lists the earlier versions of the note. Any of them can be restored; the current text is kept too.",
        icon: IconName::Clock,
        keywords: &["versions", "revisions", "restore", "undo"],
        group: Group::Note,
        run: action!(ShowHistory),
        needs: Needs::Note,
    },
    Command {
        id: "export-note",
        label: "Export note…",
        label_when_pinned: None,
        description: "Saves the note as a Markdown file, with its attachments, in a folder you choose.",
        icon: IconName::ExternalLink,
        keywords: &["markdown", "save as", "file", "share"],
        group: Group::Note,
        run: action!(ExportNote),
        needs: Needs::Note,
    },
    Command {
        id: "save",
        label: "Save note",
        label_when_pinned: None,
        description: "Saves now. Notes also save on their own a moment after typing stops.",
        icon: IconName::Check,
        keywords: &["write"],
        group: Group::Note,
        run: action!(SaveNote),
        needs: Needs::Note,
    },
    Command {
        id: "trash",
        label: "Move to trash",
        label_when_pinned: None,
        description: "Moves the note to the trash. It can be restored from there.",
        icon: IconName::Trash,
        keywords: &["delete", "remove", "discard"],
        group: Group::Note,
        run: action!(TrashNote),
        needs: Needs::LiveNote,
    },
    // View.
    Command {
        id: "edit-mode",
        label: "Edit mode",
        label_when_pinned: None,
        description: "Shows the Markdown source on its own.",
        icon: IconName::SquarePen,
        keywords: &["editor", "markdown", "source", "write"],
        group: Group::View,
        run: action!(ShowEditor),
        needs: Needs::Nothing,
    },
    Command {
        id: "split-mode",
        label: "Split mode",
        label_when_pinned: None,
        description: "Shows the Markdown source and the rendered preview side by side.",
        icon: IconName::Columns2,
        keywords: &["side by side", "both", "toggle split"],
        group: Group::View,
        run: action!(ShowSplit),
        needs: Needs::Nothing,
    },
    Command {
        id: "preview-mode",
        label: "Preview mode",
        label_when_pinned: None,
        description: "Shows the rendered note on its own.",
        icon: IconName::Eye,
        keywords: &["read", "rendered", "toggle preview"],
        group: Group::View,
        run: action!(ShowPreview),
        needs: Needs::Nothing,
    },
    Command {
        id: "sidebar",
        label: "Toggle sidebar",
        label_when_pinned: None,
        description: "Collapses the sidebar to a column of icons, or opens it again with the notebooks and tags.",
        icon: IconName::PanelLeft,
        keywords: &["hide", "show", "panel", "collapse", "expand", "icons"],
        group: Group::View,
        run: action!(ToggleSidebar),
        needs: Needs::Nothing,
    },
    Command {
        id: "focus-mode",
        label: "Focus mode",
        label_when_pinned: None,
        description: "Hides the sidebar and the note list and centres the note, for writing. The note's header shows when the pointer is near the top. Escape leaves it.",
        icon: IconName::Focus,
        keywords: &["zen", "distraction free", "writing", "full", "hide panes"],
        group: Group::View,
        run: action!(ToggleFocusMode),
        needs: Needs::Nothing,
    },
    // Go to; the library adds its places before these.
    Command {
        id: "go-to-notebook",
        label: "Go to notebook…",
        label_when_pinned: None,
        description: "Lists the notebooks; the one chosen shows in the note list.",
        icon: IconName::Notebook,
        keywords: &["notebooks", "folder", "open", "switch", "show"],
        group: Group::GoTo,
        run: Run::Pick(Picker::GoToNotebook),
        needs: Needs::Nothing,
    },
    Command {
        id: "next-note",
        label: "Next note",
        label_when_pinned: None,
        description: "Opens the note below the current one in the list.",
        icon: IconName::ArrowDown,
        keywords: &["down", "following"],
        group: Group::GoTo,
        run: action!(NextNote),
        needs: Needs::Nothing,
    },
    Command {
        id: "previous-note",
        label: "Previous note",
        label_when_pinned: None,
        description: "Opens the note above the current one in the list.",
        icon: IconName::ArrowUp,
        keywords: &["up", "back"],
        group: Group::GoTo,
        run: action!(PreviousNote),
        needs: Needs::Nothing,
    },
    // App.
    Command {
        id: "sync",
        label: "Sync now",
        label_when_pinned: None,
        description: "Sends and fetches changes now, without waiting for the next automatic sync.",
        icon: IconName::RefreshCcw,
        keywords: &["cloud", "upload", "download", "refresh"],
        group: Group::App,
        run: action!(SyncNow),
        needs: Needs::Nothing,
    },
    Command {
        id: "import-inkdrop",
        label: "Import from Inkdrop…",
        label_when_pinned: None,
        description: "Copies notes, notebooks, tags and attachments from Inkdrop. Inkdrop's files are only read.",
        icon: IconName::Upload,
        keywords: &["migrate", "copy", "notes"],
        group: Group::App,
        run: action!(ImportInkdrop),
        needs: Needs::Nothing,
    },
    Command {
        id: "export-all",
        label: "Export all notes…",
        label_when_pinned: None,
        description: "Saves every note as a Markdown file, in folders named after the notebooks, in a folder you choose.",
        icon: IconName::HardDrive,
        keywords: &["backup", "markdown", "library"],
        group: Group::App,
        run: action!(ExportLibrary),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings",
        label: "Settings",
        label_when_pinned: None,
        description: "Opens the settings in place of the notes, at the page shown last. Escape goes back to the notes.",
        icon: IconName::Settings,
        keywords: &["preferences", "options"],
        group: Group::App,
        run: action!(OpenSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-general",
        label: "Settings: General",
        label_when_pinned: None,
        description: "Opening at login, the data folder, backups and importing from Inkdrop.",
        icon: IconName::Settings,
        keywords: &[
            "preferences",
            "login",
            "startup",
            "backup",
            "data folder",
            "import",
        ],
        group: Group::App,
        run: action!(OpenGeneralSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-editor",
        label: "Settings: Editor",
        label_when_pinned: None,
        description: "The editor's font, text size and line numbers.",
        icon: IconName::Pencil,
        keywords: &["preferences", "font", "size", "monospace", "line numbers"],
        group: Group::App,
        run: action!(OpenEditorSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-appearance",
        label: "Settings: Appearance",
        label_when_pinned: None,
        description: "Light, dark, or following macOS.",
        icon: IconName::Palette,
        keywords: &["preferences", "theme", "dark mode", "light mode"],
        group: Group::App,
        run: action!(OpenAppearanceSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-sync",
        label: "Settings: Sync",
        label_when_pinned: None,
        description: "Whether notes are synced, the Worker URL and access token, and how often sync runs.",
        icon: IconName::Cloud,
        keywords: &["preferences", "cloud", "worker", "token", "conflict"],
        group: Group::App,
        run: action!(OpenSyncSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-api",
        label: "Settings: Local API",
        label_when_pinned: None,
        description: "The Inkdrop-compatible local API: its address, port, username and password, and the MCP endpoint.",
        icon: IconName::Plug,
        keywords: &[
            "preferences",
            "api",
            "mcp",
            "port",
            "password",
            "inkdrop",
            "server",
        ],
        group: Group::App,
        run: action!(OpenApiSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "quit",
        label: "Quit Kanso",
        label_when_pinned: None,
        description: "Saves open notes and quits.",
        icon: IconName::Close,
        keywords: &["exit", "close"],
        group: Group::App,
        run: action!(Quit),
        needs: Needs::Nothing,
    },
];

/// Every command, in the order the palette lists them.
pub fn all() -> &'static [Command] {
    COMMANDS
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui_kit::{AssetSource as _, TestAppContext};

    use super::{Run, all};
    use crate::assets::KansoAssets;
    use crate::workspace::OpenCommandPalette;

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for command in all() {
            assert!(seen.insert(command.id), "{} twice", command.id);
        }
    }

    #[test]
    fn labels_are_unique_and_follow_the_spec_words() {
        let mut seen = HashSet::new();
        for command in all() {
            assert!(seen.insert(command.label), "{} twice", command.label);
            let words = format!("{} {}", command.label, command.description).to_lowercase();
            assert!(!words.contains("vault"), "{}", command.id);
        }
    }

    /// Every shortcut that works anywhere in the window also has a command,
    /// so the palette is a complete list of what the keyboard can do.
    #[gpui_kit::test]
    fn every_window_wide_shortcut_has_a_command(cx: &mut TestAppContext) {
        cx.update(|cx| {
            crate::workspace::bind_keys(cx);
            crate::note_list::bind_keys(cx);
            let keymap = cx.key_bindings();
            let keymap = keymap.borrow();
            let mut checked = 0;
            for binding in keymap.bindings() {
                // Keys for one view (Enter in the list, arrows in the
                // palette) are navigation, not commands.
                if binding.predicate().is_some() || binding.action().partial_eq(&OpenCommandPalette)
                {
                    continue;
                }
                checked += 1;
                assert!(
                    all()
                        .iter()
                        .filter_map(|c| c.action())
                        .any(|a| a.partial_eq(binding.action())),
                    "no command for {}",
                    binding.action().name()
                );
            }
            assert!(checked >= 15, "{checked}");
        });
    }

    #[test]
    fn pickers_have_no_action_and_actions_build() {
        for command in all() {
            match command.run {
                Run::Action(_) => assert!(command.action().is_some()),
                Run::Pick(_) => assert!(command.action().is_none()),
            }
        }
    }

    #[test]
    fn every_icon_is_bundled() {
        for command in all() {
            let icons = [
                Some(command.icon),
                command.label_when_pinned.map(|(_, icon)| icon),
            ];
            for icon in icons.into_iter().flatten() {
                let path = icon.path();
                assert!(
                    matches!(KansoAssets.load(&path), Ok(Some(_))),
                    "{}: {path}",
                    command.id
                );
            }
        }
    }
}
