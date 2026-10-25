//! A plugin's panel in the inspector.

use std::collections::BTreeMap;

use gpui_kit::*;
use serde_json::Value;

use crate::plugins_model::PanelState;

/// The values a new panel tree starts with: what the person left in the
/// open panel's inputs.
pub fn initial_values(
    _open: &Option<PanelState>,
    values: BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    values
}

/// Shows the inspector's panel.
pub fn reveal(cx: &mut App) {
    cx.refresh_windows();
}

/// The open panel, drawn for the inspector; None when no panel is open.
pub fn element(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
    None
}
