//! What the command palette can do, apart from the places and notes that
//! come from the library (notebooks, statuses, notes). Most commands
//! dispatch the same action as their menu item or shortcut, so the palette
//! shows the shortcut the keymap really has; a few open a second list in
//! the palette to pick a notebook, tag or status from.

use gpui_kit::Action;
use gpui_kit::assets::IconName;

use crate::settings::{
    CloseSettings, OpenAccountSettings, OpenApiSettings, OpenAppearanceSettings,
    OpenEditorSettings, OpenGeneralSettings, ReloadThemes,
};
use crate::workspace::{
    ChangePassword, ExportLibrary, ExportNote, FocusNoteList, FocusSearch, ImportInkdrop, NewNote,
    NewNotebook, NewRecoveryKey, NextNote, OpenNotePicker, OpenSettings, PreviousNote, Quit,
    SaveNote, SearchAllNotes, ShowCreateAccount, ShowEditor, ShowHistory, ShowPreview, ShowSignIn,
    ShowSplit, SignOut, SyncNow, ToggleFocusMode, TogglePinned, ToggleSidebar, TrashNote,
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
    /// A tag whose notes the list shows.
    GoToTag,
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
    /// Settings are open, in place of the notes.
    Settings,
    /// This Mac is signed in to an account.
    SignedIn,
    /// This Mac isn't signed in (or was signed out by the server).
    SignedOut,
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
        id: "go-to-tag",
        label: "Go to tag…",
        label_when_pinned: None,
        description: "Lists the tags; the notes with the one chosen show in the note list.",
        icon: IconName::Hash,
        keywords: &["tags", "label", "show", "switch", "filter"],
        group: Group::GoTo,
        run: Run::Pick(Picker::GoToTag),
        needs: Needs::Nothing,
    },
    Command {
        id: "go-to-list",
        label: "Go to note list",
        label_when_pinned: None,
        description: "Moves the keyboard to the note list, where the arrow keys pick a note and Return opens it for writing.",
        icon: IconName::List,
        keywords: &["focus", "list", "notes", "keyboard", "navigate"],
        group: Group::GoTo,
        run: action!(FocusNoteList),
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
        needs: Needs::SignedIn,
    },
    Command {
        id: "sign-in",
        label: "Sign in to sync…",
        label_when_pinned: None,
        description: "Signs this Mac in to your Kanso account, to sync its notes with your other Macs, end-to-end encrypted.",
        icon: IconName::LogIn,
        keywords: &["account", "log in", "login", "cloud"],
        group: Group::App,
        run: action!(ShowSignIn),
        needs: Needs::SignedOut,
    },
    Command {
        id: "create-account",
        label: "Create account…",
        label_when_pinned: None,
        description: "Creates a Kanso account to sync notes between your Macs, end-to-end encrypted. The notes on this Mac are uploaded.",
        icon: IconName::UserPlus,
        keywords: &["sign up", "register", "sync", "cloud"],
        group: Group::App,
        run: action!(ShowCreateAccount),
        needs: Needs::SignedOut,
    },
    Command {
        id: "change-password",
        label: "Change password…",
        label_when_pinned: None,
        description: "Changes your account's password. Your other Macs ask for the new one.",
        icon: IconName::KeyRound,
        keywords: &["account", "security", "sync"],
        group: Group::App,
        run: action!(ChangePassword),
        needs: Needs::SignedIn,
    },
    Command {
        id: "new-recovery-key",
        label: "New recovery key…",
        label_when_pinned: None,
        description: "Replaces your account's recovery key. The one you saved stops working.",
        icon: IconName::FileKey,
        keywords: &["account", "security", "forgot password"],
        group: Group::App,
        run: action!(NewRecoveryKey),
        needs: Needs::SignedIn,
    },
    Command {
        id: "sign-out",
        label: "Sign out",
        label_when_pinned: None,
        description: "Signs this Mac out of your account. Notes stay on this Mac; sync stops until you sign in again.",
        icon: IconName::LogOut,
        keywords: &["account", "log out", "logout", "sync"],
        group: Group::App,
        run: action!(SignOut),
        needs: Needs::SignedIn,
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
        id: "back-to-notes",
        label: "Back to notes",
        label_when_pinned: None,
        description: "Leaves Settings for the notes as they were: the same note, with the keyboard where it was.",
        icon: IconName::ArrowLeft,
        keywords: &["close settings", "leave", "return", "notes", "done"],
        group: Group::App,
        run: action!(CloseSettings),
        needs: Needs::Settings,
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
        label: "Settings: Themes",
        label_when_pinned: None,
        description: "The UI, editor syntax and preview themes, light and dark.",
        icon: IconName::Palette,
        keywords: &[
            "preferences",
            "appearance",
            "theme",
            "color scheme",
            "colors",
            "syntax",
            "dark mode",
            "light mode",
        ],
        group: Group::App,
        run: action!(OpenAppearanceSettings),
        needs: Needs::Nothing,
    },
    Command {
        id: "reload-themes",
        label: "Reload Themes",
        label_when_pinned: None,
        description: "Reads the themes folder again, after adding or editing a theme file.",
        icon: IconName::RefreshCw,
        keywords: &["theme", "colors", "refresh"],
        group: Group::App,
        run: action!(ReloadThemes),
        needs: Needs::Nothing,
    },
    Command {
        id: "settings-account",
        label: "Settings: Account",
        label_when_pinned: None,
        description: "Your sync account: signing in, storage, devices, the password and recovery key, and how often sync runs.",
        icon: IconName::CircleUser,
        keywords: &[
            "preferences",
            "sync",
            "cloud",
            "devices",
            "recovery key",
            "conflict",
        ],
        group: Group::App,
        run: action!(OpenAccountSettings),
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

    /// Shortcuts for one place are skipped above, but some of them leave or
    /// move between places, and those have commands too: Escape leaves
    /// Settings (the palette lists it only while they are open).
    #[test]
    fn leaving_and_moving_have_commands() {
        use super::Needs;
        use crate::settings::CloseSettings;
        use crate::workspace::FocusNoteList;
        let find = |action: &dyn gpui_kit::Action| {
            all()
                .iter()
                .find(|c| c.action().is_some_and(|a| a.partial_eq(action)))
        };
        let back = find(&CloseSettings).expect("a command leaves Settings");
        assert_eq!(back.needs, Needs::Settings);
        assert_eq!(back.label, "Back to notes");
        assert!(find(&FocusNoteList).is_some(), "a command goes to the list");
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
