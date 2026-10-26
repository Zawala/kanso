//! The built-in themes (`themes/`, compiled into `kanso-engine`), fitted as
//! the desktop fits them for its own panes. Each is fitted once, the first
//! time it is asked for, and kept.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use kanso_engine::theme::Tokens;
use kanso_engine::theme::builtin::{BUILTIN, id_of};
use kanso_engine::theme::file::ThemeFile;

use crate::boundary::held;
use crate::errors::KansoError;
use crate::model::{BuiltinTheme, ThemeTokens};
use crate::theme_tokens;

/// A theme as fitted: who made it, the rows the shell reads and the fitted
/// styles the highlighter's runs are folded with.
pub(crate) struct Fitted {
    id: String,
    file: ThemeFile,
    pub(crate) tokens: ThemeTokens,
    pub(crate) fitted: Tokens,
}

impl Fitted {
    fn row(&self) -> BuiltinTheme {
        BuiltinTheme {
            id: self.id.clone(),
            name: self.file.name.clone(),
            author: self.file.author.clone(),
            license: self.file.license.clone(),
            source: self.file.source.clone(),
            dark: self.tokens.dark,
            tokens: self.tokens.clone(),
        }
    }
}

static FITTED: LazyLock<Mutex<HashMap<&'static str, Arc<Fitted>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The built-in theme `id` (its file name without `.json`), fitted.
pub(crate) fn get(id: &str) -> Result<Arc<Fitted>, KansoError> {
    let (file, text) = BUILTIN
        .iter()
        .find(|(file, _)| id_of(file) == id)
        .ok_or(KansoError::NotFound)?;
    let mut fitted = held(&FITTED);
    if let Some(found) = fitted.get(file) {
        return Ok(found.clone());
    }
    let parsed = ThemeFile::parse(text)
        .map_err(|err| KansoError::invalid(format!("The theme {file} {err}.")))?;
    let (tokens, fit) = theme_tokens::fit_file(&parsed);
    let theme = Arc::new(Fitted {
        id: id.to_owned(),
        file: parsed,
        tokens,
        fitted: fit,
    });
    fitted.insert(file, theme.clone());
    Ok(theme)
}

/// Every built-in theme, in the order of the desktop's Themes page.
pub(crate) fn all() -> Result<Vec<BuiltinTheme>, KansoError> {
    BUILTIN
        .iter()
        .map(|(file, _)| get(id_of(file)).map(|theme| theme.row()))
        .collect()
}
