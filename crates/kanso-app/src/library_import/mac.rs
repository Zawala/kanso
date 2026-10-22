//! The one macOS access grant Apple Notes needs.
//!
//! A non-sandboxed app can still be stopped by the privacy gate around
//! `group.com.apple.notes`. A user-chosen panel grants read access without
//! making the person find or understand the group container; the panel opens
//! directly on that folder. The folder grant also covers NoteStore.sqlite's
//! WAL, which is part of the live database.

use std::cell::Cell;
use std::path::{Path, PathBuf};

use block2::RcBlock;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponse, NSModalResponseOK, NSOpenPanel};
use objc2_foundation::{NSString, NSURL};

/// Opens the standard open panel on the Apple Notes folder and sends that
/// folder once the person allows access.
pub(super) fn choose_apple_notes_folder(
    root: &Path,
    done: async_channel::Sender<Option<PathBuf>>,
) -> Result<(), String> {
    let mtm = MainThreadMarker::new()
        .ok_or("the Apple Notes access panel has to open on the main thread")?;
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseFiles(false);
    panel.setCanChooseDirectories(true);
    panel.setAllowsMultipleSelection(false);
    panel.setCanCreateDirectories(false);
    panel.setResolvesAliases(false);
    panel.setTitle(Some(&NSString::from_str("Allow Kanso to read Apple Notes")));
    panel.setMessage(Some(&NSString::from_str(
        "macOS protects the Notes library. The right folder is open below; click Allow. Kanso only reads your notes.",
    )));
    panel.setPrompt(Some(&NSString::from_str("Allow")));
    if let Some(url) = NSURL::from_directory_path(root) {
        panel.setDirectoryURL(Some(&url));
    }

    let done = Cell::new(Some(done));
    let handler = RcBlock::new({
        let panel = panel.clone();
        move |response: NSModalResponse| {
            let picked = (response == NSModalResponseOK)
                .then(|| panel.URL().and_then(|url| url.to_file_path()))
                .flatten();
            if let Some(done) = done.take() {
                done.try_send(picked).ok();
            }
        }
    });
    panel.beginWithCompletionHandler(&handler);
    Ok(())
}
