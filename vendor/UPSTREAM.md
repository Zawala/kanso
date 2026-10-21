# Proposal: an icon resolver hook for gpui-component

This is a change to offer to [GPUI Kit](https://github.com/longbridge/gpui-kit)
(`crates/component`), written against gpui-component 0.7.3. It hasn't been
sent anywhere.

## Summary

Add an app-level hook that maps an icon's asset path to the path to draw,
asked by every `Icon` drawn from a path and by the two places that draw a
check mark with `svg()` directly. It lets an application restyle every icon
in its windows, the library's own included, and switch styles live.

## Motivation

An app that offers icon themes (a different icon set, chosen at run time)
can draw its own icons from any path it likes, but the icons the library
builds inside its controls (a checkbox's check, a select's chevron, the
close button of a dialog or notification, a date picker's arrows and
the rest) always come from `IconName::path()`. GPUI caches each rasterized
SVG by path and size, per window, with no way to evict, so swapping the
bytes behind a path doesn't redraw it either. A resolver that returns a new
path for a new style is the one way to restyle those icons, and it switches
them at once: a new path is a cache miss, an unchanged one is a hit.

## API

```rust
/// Sets the hook every `Icon` drawn from a path asks for the path to draw.
pub fn set_icon_resolver(
    cx: &mut App,
    resolve: impl Fn(&str, &App) -> Option<SharedString> + 'static,
);

/// The path to draw for the icon at `path`: the resolver's answer, or `path`.
pub fn resolve_icon_path(path: SharedString, cx: &App) -> SharedString;
```

- `None` (or no resolver) keeps today's behaviour exactly.
- Icons given bytes with `Icon::data` aren't asked.
- The resolver runs for every icon drawn, so it should be cheap (a map
  lookup); after changing what it returns, the app calls
  `cx.refresh_windows()`.

## Tests to add upstream

- `Icon::new(IconName::Search)` rendered with a resolver that maps
  `icons/search.svg` to `icons/alt/search.svg` paints the new path; without
  a resolver, the original.
- A checked `Checkbox` asks the resolver for `icons/check.svg`.

(Kanso's `switching_icon_theme_redraws_controls_and_kanso_icons_in_one_window`
does both against this patch.)

## Diff

```diff
diff --color -ru gpui-component-0.7.3/src/checkbox.rs gpui-component/src/checkbox.rs
--- gpui-component-0.7.3/src/checkbox.rs	2006-07-24 03:21:28
+++ gpui-component/src/checkbox.rs	2026-10-21 23:57:21
@@ -216,7 +216,8 @@
         })
         .text_color(color)
         .when(opacity > 0., |this| {
-            this.path(IconName::Check.path()).opacity(opacity)
+            this.path(crate::resolve_icon_path(IconName::Check.path(), cx))
+                .opacity(opacity)
         })
 }
 
diff --color -ru gpui-component-0.7.3/src/icon.rs gpui-component/src/icon.rs
--- gpui-component-0.7.3/src/icon.rs	2006-07-24 03:21:28
+++ gpui-component/src/icon.rs	2026-10-21 23:57:21
@@ -1,8 +1,9 @@
+use std::rc::Rc;
 use std::sync::Arc;
 
 use crate::{ActiveTheme, Sizable, Size};
 use gpui::{
-    AnyElement, App, AppContext, Context, Entity, Hsla, IntoElement, Pixels, Radians, Render,
+    AnyElement, App, AppContext, Context, Entity, Global, Hsla, IntoElement, Pixels, Radians, Render,
     RenderOnce, SharedString, StyleRefinement, Styled, Svg, Transformation, Window,
     prelude::FluentBuilder as _, svg,
 };
@@ -74,7 +75,44 @@
     }
 }
 
+/// Draws icons from other files: the hook an app sets to restyle every
+/// icon by its asset path, the library's own icons included.
+///
+/// An [`Icon`] given a path (`icons/check.svg`) asks the resolver, when one
+/// is set, as it draws; a path the resolver returns is drawn in its place,
+/// and `None` keeps the path as it was. GPUI keeps an icon it has drawn by
+/// its path, so returning a new path for a new look (say, with the look's
+/// name in it) shows the new look at once, with no cache to clear: the
+/// app's [`gpui::AssetSource`] loads the new path. Icons given SVG bytes
+/// (`Icon::data`) aren't asked.
+///
+/// The resolver runs for every icon drawn, so it should be quick: a map
+/// lookup, say. Set it with [`set_icon_resolver`]; call
+/// [`gpui::App::refresh_windows`] when what it returns changes.
 #[derive(Clone)]
+pub struct IconResolver(Rc<dyn Fn(&str, &App) -> Option<SharedString>>);
+
+impl Global for IconResolver {}
+
+/// Sets the hook every [`Icon`] drawn from a path asks for the path to
+/// draw; see [`IconResolver`].
+pub fn set_icon_resolver(
+    cx: &mut App,
+    resolve: impl Fn(&str, &App) -> Option<SharedString> + 'static,
+) {
+    cx.set_global(IconResolver(Rc::new(resolve)));
+}
+
+/// The path to draw for the icon at `path`: the [`IconResolver`]'s answer,
+/// or `path` itself.
+pub fn resolve_icon_path(path: SharedString, cx: &App) -> SharedString {
+    match cx.try_global::<IconResolver>() {
+        Some(resolver) if !path.is_empty() => (resolver.0)(&path, cx).unwrap_or(path),
+        _ => path,
+    }
+}
+
+#[derive(Clone)]
 pub(crate) enum IconSource {
     Path(SharedString),
     Data(Arc<[u8]>),
@@ -172,6 +210,14 @@
         self
     }
 
+    /// The icon with its path resolved by the app's [`IconResolver`].
+    fn resolved(mut self, cx: &App) -> Self {
+        if let IconSource::Path(path) = self.source {
+            self.source = IconSource::Path(resolve_icon_path(path, cx));
+        }
+        self
+    }
+
     fn into_svg(self, text_size: Pixels, fallback_color: Hsla) -> Svg {
         let text_color = self.text_color.unwrap_or(fallback_color);
         let has_base_size = self.style.size.width.is_some() || self.style.size.height.is_some();
@@ -220,9 +266,10 @@
 }
 
 impl RenderOnce for Icon {
-    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
+    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
         let text_size = window.text_style().font_size.to_pixels(window.rem_size());
-        self.into_svg(text_size, window.text_style().color)
+        self.resolved(cx)
+            .into_svg(text_size, window.text_style().color)
     }
 }
 
@@ -235,7 +282,9 @@
 impl Render for Icon {
     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
         let text_size = window.text_style().font_size.to_pixels(window.rem_size());
-        self.clone().into_svg(text_size, cx.theme().foreground)
+        self.clone()
+            .resolved(cx)
+            .into_svg(text_size, cx.theme().foreground)
     }
 }
 
diff --color -ru gpui-component-0.7.3/src/questionnaire/components.rs gpui-component/src/questionnaire/components.rs
--- gpui-component-0.7.3/src/questionnaire/components.rs	2006-07-24 03:21:28
+++ gpui-component/src/questionnaire/components.rs	2026-10-21 23:57:57
@@ -864,6 +864,7 @@
             .is_some_and(|focus_handle| focus_handle.is_focused(window));
         let has_children = !self.children.is_empty();
 
+        let check_path = crate::resolve_icon_path(IconName::Check.path(), cx);
         let default_indicator = || {
             div()
                 .relative()
@@ -890,7 +891,7 @@
                     this.child(
                         svg()
                             .size(metrics.indicator_check_size)
-                            .path(IconName::Check.path())
+                            .path(check_path.clone())
                             .text_color(colors.primary_foreground),
                     )
                 })
```
