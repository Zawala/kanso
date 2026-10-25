//! A plugin's panel: a tree of nodes Kanso draws, never markup. There is no
//! password field, no full-window surface and nothing that can cover
//! Kanso's own title bar or dialogs. [`crate::panel::check`] holds a tree to
//! its caps before it is drawn.

use serde::{Deserialize, Serialize};

/// The most nodes a panel may have.
pub const MAX_NODES: usize = 500;
/// The deepest a panel may nest.
pub const MAX_DEPTH: usize = 8;
/// The longest string anywhere in a panel, in bytes.
pub const MAX_STRING: usize = 8 * 1024;
/// The most options a select may have.
pub const MAX_OPTIONS: usize = 100;

/// One node. Unknown types and unknown fields are refused.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Node {
    Text {
        value: String,
        #[serde(default)]
        style: TextStyle,
    },
    Heading {
        value: String,
    },
    Row {
        children: Vec<Node>,
        #[serde(default)]
        gap: Option<f32>,
    },
    Column {
        children: Vec<Node>,
        #[serde(default)]
        gap: Option<f32>,
    },
    /// Lines of text, or rows.
    List {
        items: Vec<Node>,
    },
    Button {
        id: String,
        label: String,
        #[serde(default)]
        tone: Tone,
    },
    /// Plain text; never a password field.
    Input {
        id: String,
        label: String,
        #[serde(default)]
        value: String,
        #[serde(default)]
        placeholder: String,
    },
    Toggle {
        id: String,
        label: String,
        #[serde(default)]
        value: bool,
    },
    Select {
        id: String,
        label: String,
        options: Vec<String>,
        #[serde(default)]
        value: Option<String>,
    },
    /// An attachment of a note the plugin may read.
    Image {
        #[serde(rename = "attachmentId")]
        attachment_id: String,
    },
    /// An HTTPS link, shown in full and opened in the browser.
    Link {
        label: String,
        url: String,
    },
    Badge {
        value: String,
    },
    /// 0 to 1.
    Progress {
        value: f32,
    },
    Divider,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextStyle {
    #[default]
    Body,
    Muted,
    Caption,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    #[default]
    Normal,
    Primary,
    Danger,
}

/// A panel as the app shows it: the plugin's tree under Kanso's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    pub plugin_id: String,
    /// The plugin's name and version, in Kanso's frame.
    pub plugin_name: String,
    pub plugin_version: String,
    pub root: Node,
}
