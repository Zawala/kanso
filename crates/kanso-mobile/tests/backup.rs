//! The library's folder is kept out of device backups, on every open.

mod common;

use std::sync::Arc;

use kanso_mobile::diagnostics::backup_excluded;
use kanso_mobile::{Library, MemorySecrets};
use kanso_sync::MemoryServer;
use proptest::prelude::*;

fn open(path: &std::path::Path) -> Library {
    Library::open_for_test(
        path,
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Whatever folder a library is opened in, that folder is excluded from
    /// backups afterwards (on Apple targets; nowhere else has the flag), and
    /// the folder around it is left as it was. Opening again changes
    /// nothing.
    #[test]
    fn prop_an_opened_library_folder_is_excluded_and_its_parent_is_not(
        name in "[a-z]{1,8}",
        depth in 1usize..3,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let mut folder = dir.path().to_path_buf();
        for level in 0..depth {
            folder.push(format!("{name}{level}"));
        }
        let before = backup_excluded(dir.path());
        let library = open(&folder.join("library.db"));
        let expected = if cfg!(target_vendor = "apple") { Some(true) } else { None };
        prop_assert_eq!(backup_excluded(&folder), expected);
        // The folder around it, which the test made, is not touched.
        prop_assert_eq!(backup_excluded(dir.path()), before);
        if cfg!(target_vendor = "apple") {
            prop_assert_eq!(before, Some(false));
        }
        drop(library);
        let again = open(&folder.join("library.db"));
        prop_assert_eq!(backup_excluded(&folder), expected);
        drop(again);
    }
}

// no-prop-test: a folder that is not there.
#[test]
fn a_folder_that_does_not_exist_reports_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(backup_excluded(&dir.path().join("nowhere")), None);
}
