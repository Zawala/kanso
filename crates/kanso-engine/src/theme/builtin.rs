//! The built-in color themes: `themes/*.json`, the kanso-themes submodule,
//! compiled in by this crate's `build.rs` so the desktop and the phones draw
//! from one list in one order.

include!(concat!(env!("OUT_DIR"), "/builtin_themes.rs"));

/// A built-in theme's id: its file name without `.json`.
pub fn id_of(file: &str) -> &str {
    file.trim_end_matches(".json")
}

#[cfg(test)]
mod tests {
    use super::{BUILTIN, id_of};
    use crate::theme::file::ThemeFile;

    // no-prop-test: fixed files, all of them read; a property over them
    // would be this loop.
    #[test]
    fn every_built_in_theme_parses_and_kanso_is_first() {
        assert!(BUILTIN.len() > 20, "{} themes", BUILTIN.len());
        assert_eq!(BUILTIN[0].0, "kanso-light.json");
        assert_eq!(BUILTIN[1].0, "kanso-dark.json");
        let mut seen = std::collections::HashSet::new();
        for (file, text) in BUILTIN {
            assert!(seen.insert(*file), "{file} is listed twice");
            ThemeFile::parse(text).unwrap_or_else(|err| panic!("{file}: {err}"));
        }
        assert_eq!(id_of("nord.json"), "nord");
    }
}
