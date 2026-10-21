//! What can be done to one note, as a menu: the same items, in the same
//! order, from a right click on its row in the list and from the editor's
//! "More" button. The workspace carries each choice out
//! (`Workspace::run_note_command`).

use std::rc::Rc;

use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::*;
use kanso_core::NoteStatus;

/// One item of the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteCommand {
    TogglePinned,
    Duplicate,
    SetStatus(NoteStatus),
    ShowHistory,
    Export,
    Publish,
    Trash,
}

/// What a choice does: given the command, carries it out.
pub type Run = Rc<dyn Fn(NoteCommand, &mut Window, &mut App)>;

/// Adds the items for a note that is `pinned` or not and has `status` to
/// `menu`, each calling `run` with its command.
pub fn items(
    menu: PopupMenu,
    pinned: bool,
    status: NoteStatus,
    run: Run,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let item = |label: &'static str, command: NoteCommand| {
        let run = run.clone();
        PopupMenuItem::new(label).on_click(move |_, window, cx| run(command, window, cx))
    };
    let statuses = run.clone();
    menu.item(item(
        if pinned { "Unpin Note" } else { "Pin Note" },
        NoteCommand::TogglePinned,
    ))
    .item(item("Duplicate", NoteCommand::Duplicate))
    .submenu("Status", window, cx, move |mut menu, _, _| {
        for option in NoteStatus::ALL {
            let run = statuses.clone();
            menu = menu.item(
                PopupMenuItem::new(option.label())
                    .checked(option == status)
                    .on_click(move |_, window, cx| run(NoteCommand::SetStatus(option), window, cx)),
            );
        }
        menu
    })
    .separator()
    .item(item("Show History…", NoteCommand::ShowHistory))
    .item(item("Export Note…", NoteCommand::Export))
    .item(item("Publish to the Web…", NoteCommand::Publish))
    .separator()
    .item(item("Move to Trash", NoteCommand::Trash))
}
