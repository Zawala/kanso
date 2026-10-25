//! Settings › Plugins.

use gpui_kit::*;

pub struct PluginsPage {}

impl PluginsPage {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        PluginsPage {}
    }

    /// Brings the page up to date each time it shows.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        cx.notify();
    }
}

impl Render for PluginsPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
