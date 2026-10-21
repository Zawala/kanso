//! An icon set the gallery lists, and Iconify's list of sets as it comes
//! (`icon_gallery::parse_collections` reads one into the other).

use serde::Deserialize;

/// One icon set the gallery lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconSet {
    /// Iconify's name for it, and the pack's ID once installed.
    pub prefix: String,
    pub name: String,
    pub total: u32,
    pub author: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    /// A few of its icons' names.
    pub samples: Vec<String>,
}

/// One set in Iconify's list.
#[derive(Deserialize)]
pub struct RawSet {
    pub name: String,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub author: Option<RawNamed>,
    #[serde(default)]
    pub license: Option<RawLicense>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub samples: Vec<String>,
    #[serde(default)]
    pub palette: bool,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Deserialize)]
pub struct RawNamed {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RawLicense {
    pub title: String,
}
