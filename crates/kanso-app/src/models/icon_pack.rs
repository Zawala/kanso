//! An icon pack as Kanso keeps it, and its `pack.json` as written
//! (`icon_packs` reads them).

use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

/// One icon of a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackIcon {
    /// The file's name without `.svg`: lowercase letters, digits and dashes.
    pub name: String,
    pub label: String,
    /// More words the search finds it by, spaced.
    pub keywords: String,
    /// Shared between an icon and the other names it goes by.
    pub svg: Arc<[u8]>,
}

/// A pack of icons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconPack {
    /// The folder's name.
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub license: Option<String>,
    /// The Iconify set the gallery installed it from, if it did.
    pub iconify: Option<String>,
    /// In name order.
    pub icons: Vec<PackIcon>,
    /// How many of the app's own icons it draws in their place, counted
    /// once when it is read.
    pub replaces: usize,
}

impl IconPack {
    pub fn icon(&self, name: &str) -> Option<&PackIcon> {
        self.icons
            .binary_search_by(|icon| icon.name.as_str().cmp(name))
            .ok()
            .map(|ix| &self.icons[ix])
    }
}

/// A pack's `pack.json`.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Manifest {
    pub name: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub source: Source,
    pub icons: HashMap<String, IconWords>,
}

/// Where a pack came from, as the gallery writes it.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Source {
    pub iconify: Option<String>,
}

/// An icon's label and search words in `pack.json`.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct IconWords {
    pub label: Option<String>,
    pub keywords: Option<String>,
}
