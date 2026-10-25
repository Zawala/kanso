//! The plugin sheets: installing or updating, reviewing a batch, and
//! trusting a developer folder.

use gpui_kit::*;

use crate::plugins_model::InstallPlan;

/// The install (or update) sheet for `plan`.
pub fn open_install(_plan: InstallPlan, _window: &mut Window, _cx: &mut App) {}

/// The review sheet for the batch waiting in [`crate::plugins::Plugins`].
pub fn open_review(_window: &mut Window, _cx: &mut App) {}
