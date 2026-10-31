//! Sticky notes: a note in a small window of its own (`note_window.rs`)
//! that stays above the other windows, with only its text, in Preview so
//! code blocks' Copy buttons are a click away. "Open as Sticky" in the
//! note's menu, the palette and ⌘⌥O open one; `kanso sticky` makes a note
//! and opens it as one (`sticky_cli.rs`).
//!
//! Staying on top is up to the window system:
//!
//! - macOS and Windows: a pop-up window, above the others.
//! - X11 (and Kanso run under XWayland, `KANSO_BACKEND=x11` or Settings ›
//!   General): `_NET_WM_STATE_ABOVE` and `_NET_WM_STATE_STICKY` asked of
//!   the window manager, so the sticky shows on every workspace too.
//! - Wayland compositors with layer-shell (KDE, Hyprland, Sway): a surface
//!   on the top layer. The compositor places it, so it opens where it was
//!   left and can't be dragged.
//! - GNOME on Wayland lets no app keep a window on top: the sticky is an
//!   ordinary small window, and says once how to pin it.

use anyhow::Result;
use gpui_kit::*;
use kanso_core::DocType;
use kanso_engine::capture::{save_capture, split_capture};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use crate::config::StickyColor;
use crate::note_window::Frame;

/// A new sticky's size, in points: a few lines of a snippet.
pub const SIZE: (f32, f32) = (300., 220.);
/// The smallest a sticky gets: its header and a line or two.
pub const MIN_SIZE: (f32, f32) = (200., 120.);
/// The sticky's header: its title, where it is dragged, and its buttons.
pub const HEADER_HEIGHT: f32 = 30.;
/// How far a new sticky with nowhere to go sits from the display's corner,
/// and from the sticky before it.
const MARGIN: f32 = 32.;
const STAGGER: f32 = 28.;
/// The notebook `kanso sticky` saves to when no other is named.
pub const BOOK_NAME: &str = "Stickies";
/// What a sticky says once where Kanso can't keep it on top.
pub const ON_TOP_HINT: &str = "GNOME doesn't let apps keep a window on top. Press Alt+Space and choose Always on Top, or turn on Run under XWayland in Settings › General and restart Kanso.";

/// The header's paper: strong enough to read as the note's colour.
pub fn paper(color: StickyColor, dark: bool) -> Hsla {
    let (h, s) = hue(color);
    if dark {
        hsla(h, s * 0.45, 0.26, 1.)
    } else {
        hsla(h, s, 0.80, 1.)
    }
}

/// A wash of the colour laid over the text, so the whole note reads as
/// paper of that colour without the text views knowing.
pub fn wash(color: StickyColor, dark: bool) -> Hsla {
    let (h, s) = hue(color);
    if dark {
        hsla(h, s * 0.6, 0.5, 0.07)
    } else {
        hsla(h, s, 0.6, 0.12)
    }
}

/// Hue and saturation, each 0 to 1.
fn hue(color: StickyColor) -> (f32, f32) {
    match color {
        StickyColor::Yellow => (48. / 360., 0.95),
        StickyColor::Blue => (208. / 360., 0.85),
        StickyColor::Green => (130. / 360., 0.55),
        StickyColor::Pink => (335. / 360., 0.85),
        StickyColor::Purple => (268. / 360., 0.65),
        StickyColor::Gray => (220. / 360., 0.08),
    }
}

/// Where a new sticky opened with nowhere to go lands: the top right of
/// `display`, each after the first `open` ones a step down and left so
/// none hides another. Pure.
pub fn corner_frame(display: Frame, open: usize) -> Frame {
    let step = STAGGER * (open % 8) as f32;
    let x = display.0 + display.2 - SIZE.0 - MARGIN - step;
    let y = display.1 + MARGIN + step;
    (x.max(display.0), y, SIZE.0, SIZE.1)
}

/// The frame a sticky opened from a window at `from` gets: its own size,
/// near that window's top right. Pure.
pub fn beside(from: Frame) -> Frame {
    (
        from.0 + (from.2 - SIZE.0 - MARGIN).max(0.),
        from.1 + MARGIN,
        SIZE.0,
        SIZE.1,
    )
}

/// Whether a sticky should try a layer-shell surface: on Wayland, outside
/// GNOME, whose compositor has none (trying there would only fail).
#[cfg(target_os = "linux")]
pub fn tries_layer_shell(cx: &App) -> bool {
    cx.compositor_name() == "Wayland" && !is_gnome()
}

#[cfg(target_os = "linux")]
fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|desktop| {
            desktop
                .split(':')
                .any(|part| part.eq_ignore_ascii_case("gnome"))
        })
        .unwrap_or(false)
}

/// Whether Kanso can keep its windows on top in this session by itself.
pub fn keeps_on_top(layer_shell: bool, cx: &App) -> bool {
    if cfg!(target_os = "linux") {
        layer_shell || cx.compositor_name() == "X11"
    } else {
        true
    }
}

/// The window kind a sticky asks for at `bounds`.
pub fn window_kind(bounds: Bounds<Pixels>, layer_shell: bool) -> WindowKind {
    #[cfg(target_os = "linux")]
    if layer_shell {
        use gpui_kit::layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions};
        return WindowKind::LayerShell(LayerShellOptions {
            namespace: "kanso-sticky".into(),
            layer: Layer::Top,
            anchor: Anchor::TOP | Anchor::LEFT,
            exclusive_zone: None,
            exclusive_edge: None,
            margin: Some((bounds.origin.y, px(0.), px(0.), bounds.origin.x)),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
        });
    }
    let _ = (bounds, layer_shell);
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        WindowKind::PopUp
    } else {
        WindowKind::Normal
    }
}

/// Asks the window manager to keep `window` above the others and on every
/// workspace, at `frame` (in points): window managers such as GNOME's
/// place a new window themselves, wherever it asked to be, and grow it by
/// its shadow once that is drawn. Only X11 has a way to ask; elsewhere the
/// window's kind did what could be done.
pub fn stay_on_top(window: &Window, frame: Frame, cx: &App) {
    #[cfg(target_os = "linux")]
    if cx.compositor_name() == "X11" {
        let scale = window.scale_factor();
        let pixels = |points: f32| (points * scale).round() as i32;
        let frame = [frame.0, frame.1, frame.2, frame.3].map(pixels);
        if let Err(err) = x11::stay_on_top(window, frame) {
            eprintln!("kanso: couldn't keep the sticky note on top: {err:#}");
        }
    }
    let _ = (window, frame, cx);
}

#[cfg(target_os = "linux")]
mod x11 {
    use anyhow::{Context as _, Result, bail};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use x11rb::connection::Connection as _;
    use x11rb::protocol::xproto::{ClientMessageEvent, ConnectionExt as _, EventMask};

    /// `_NET_WM_STATE_ADD`, and the source "a normal application" (EWMH).
    const ADD: u32 = 1;
    const FROM_APPLICATION: u32 = 1;
    /// `_NET_MOVERESIZE_WINDOW`'s first word: static gravity (the position
    /// is the window's own, not its frame's), from an application, and
    /// which values are given. Two requests: GNOME ignores the position of
    /// one that gives the size too.
    const MOVE: u32 = 10 | 0b0011 << 8 | FROM_APPLICATION << 12;
    const RESIZE: u32 = 10 | 0b1100 << 8 | FROM_APPLICATION << 12;

    /// Sends the root window the EWMH requests to add `_NET_WM_STATE_ABOVE`
    /// and `_NET_WM_STATE_STICKY` to `window`'s state, and to put it at
    /// `[x, y, width, height]` (pixels). On a connection of its own: GPUI's
    /// isn't reachable, and any client may ask these.
    pub fn stay_on_top(window: &gpui_kit::Window, [x, y, width, height]: [i32; 4]) -> Result<()> {
        let handle = HasWindowHandle::window_handle(window).context("the window has no handle")?;
        let RawWindowHandle::Xcb(handle) = handle.as_raw() else {
            bail!("not an X11 window");
        };
        let xid = handle.window.get();
        let (conn, screen) = x11rb::connect(None).context("connecting to the X server")?;
        let root = conn.setup().roots[screen].root;
        let atom =
            |name: &[u8]| -> Result<u32> { Ok(conn.intern_atom(false, name)?.reply()?.atom) };
        let state = atom(b"_NET_WM_STATE")?;
        let above = atom(b"_NET_WM_STATE_ABOVE")?;
        let sticky = atom(b"_NET_WM_STATE_STICKY")?;
        let move_to = atom(b"_NET_MOVERESIZE_WINDOW")?;
        let to_root = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
        let event =
            ClientMessageEvent::new(32, xid, state, [ADD, above, sticky, FROM_APPLICATION, 0]);
        conn.send_event(false, root, to_root, event)?;
        for data in [
            [MOVE, x as u32, y as u32, 0, 0],
            [RESIZE, 0, 0, width as u32, height as u32],
        ] {
            let event = ClientMessageEvent::new(32, xid, move_to, data);
            conn.send_event(false, root, to_root, event)?;
        }
        conn.flush()?;
        Ok(())
    }
}

/// The notebook stickies go to: the first named [`BOOK_NAME`], ignoring
/// case, made when there is none. Returns its ID.
pub fn stickies_book(store: &Store, origin: Origin) -> Result<String> {
    if let Some(book) = store
        .books()?
        .into_iter()
        .find(|book| book.name.trim().eq_ignore_ascii_case(BOOK_NAME))
    {
        return Ok(book.id);
    }
    let made = store.put(
        DocType::Book,
        json!({"name": BOOK_NAME, "parentBookId": null}),
        Put::Upsert,
        origin,
    )?;
    Ok(made.id)
}

/// Saves `text` as a note in `book`, or in the Stickies notebook, and
/// returns its ID; `None` when the text is empty. The first line is its
/// title, as for a capture; a single line is its text too, so the sticky
/// shows it and Copy copies it.
pub fn save_sticky(
    store: &Store,
    text: &str,
    book: Option<&str>,
    origin: Origin,
) -> Result<Option<String>> {
    let book = match book {
        Some(book) => book.to_owned(),
        None => stickies_book(store, origin)?,
    };
    let text = match split_capture(text) {
        Some((_, body)) if body.is_empty() => format!("{0}\n\n{0}", text.trim()),
        _ => text.to_owned(),
    };
    Ok(save_capture(store, &text, Some(&book), origin)?)
}

#[cfg(test)]
mod tests {
    // Not `super::*`: GPUI's `test` would stand in for the standard one.
    use kanso_store::{Origin, Store};

    use super::{BOOK_NAME, Frame, MARGIN, SIZE, beside, corner_frame, save_sticky};

    const LAPTOP: Frame = (0., 0., 1440., 900.);

    #[test]
    fn a_new_sticky_lands_in_the_top_right_corner() {
        let first = corner_frame(LAPTOP, 0);
        assert_eq!(first, (1440. - SIZE.0 - MARGIN, MARGIN, SIZE.0, SIZE.1));
        let second = corner_frame(LAPTOP, 1);
        assert!(second.0 < first.0 && second.1 > first.1);
        // On a display beside the main one, it stays on that display.
        let beside_display = corner_frame((1440., 0., 1920., 1080.), 0);
        assert!(beside_display.0 > 1440.);
    }

    #[test]
    fn a_sticky_from_a_window_sits_inside_its_top_right() {
        let (x, y, w, h) = beside((100., 80., 780., 820.));
        assert_eq!((w, h), SIZE);
        assert!(x > 100. && x + w <= 100. + 780.);
        assert!(y > 80.);
    }

    #[test]
    fn stickies_go_to_one_stickies_notebook() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let first = save_sticky(&store, "docker ps -a", None, Origin::Api)
            .unwrap()
            .unwrap();
        let second = save_sticky(&store, "kubectl get pods", None, Origin::Api)
            .unwrap()
            .unwrap();
        let books: Vec<_> = store
            .books()
            .unwrap()
            .into_iter()
            .filter(|b| b.name == BOOK_NAME)
            .collect();
        assert_eq!(books.len(), 1);
        for id in [&first, &second] {
            let note = store.note(id).unwrap().unwrap();
            assert_eq!(note.book_id, books[0].id);
        }
        assert_eq!(
            save_sticky(&store, "  \n ", None, Origin::Api).unwrap(),
            None
        );
        // One line is the title and the text.
        let note = store.note(&first).unwrap().unwrap();
        assert_eq!(
            (note.title.as_str(), note.body.as_str()),
            ("docker ps -a", "docker ps -a")
        );
    }
}
