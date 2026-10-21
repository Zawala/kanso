//! An icon set in Iconify's JSON format: the parts Kanso draws
//! (`icon_packs::iconify_svgs` turns it into SVGs).

use std::collections::HashMap;

use serde::Deserialize;

/// An `icons.json`, or an answer from Iconify's API.
#[derive(Debug, Deserialize)]
pub struct IconifyJson {
    pub icons: HashMap<String, IconifyIcon>,
    #[serde(default)]
    pub aliases: HashMap<String, IconifyAlias>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub left: Option<f32>,
    #[serde(default)]
    pub top: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct IconifyIcon {
    pub body: String,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub left: Option<f32>,
    #[serde(default)]
    pub top: Option<f32>,
    #[serde(default)]
    pub hidden: bool,
}

/// Another name for an icon; one that turns or flips it is left out.
#[derive(Debug, Deserialize)]
pub struct IconifyAlias {
    pub parent: String,
    #[serde(default)]
    pub rotate: Option<u8>,
    #[serde(default, rename = "hFlip")]
    pub h_flip: bool,
    #[serde(default, rename = "vFlip")]
    pub v_flip: bool,
    #[serde(default)]
    pub hidden: bool,
}
