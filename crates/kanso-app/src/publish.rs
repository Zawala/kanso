//! The Publish sheet: a note's web page at a link anyone can open, sealed on
//! this device so the sync server keeps only a copy it can't read
//! (docs/ACCOUNTS.md › Published notes).
//!
//! The page is the note's web page export (`export_html::page`), images
//! inside it, in the preview's theme when that is light and in Kanso's light
//! theme otherwise. `kanso_sync::share` seals and uploads it; the link and
//! its key are kept in the note (`kansoPublish`), so the account's other
//! devices can update or unpublish it. Publishing needs an account: signed out,
//! the sheet says so and offers to sign in.

use std::sync::Arc;

use chrono::{DateTime, Datelike as _, Timelike as _};
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, StyledExt as _,
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use kanso_sync::ApiError;
use kanso_sync::share::{self, Expiry, KansoPublish, Page, ShareError};

use crate::clock::Clock;
use crate::editor::NoteEditor;
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [PublishNote]);

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 480.;

/// What the sheet publishes: the open note as typed.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub body: String,
    pub rev: String,
    pub publish: Option<KansoPublish>,
}

/// What the sheet shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Publishing needs an account.
    SignedOut,
    Unpublished,
    Published,
}

/// What the sheet shows for an account state and a note. Pure.
pub fn phase(signed_in: bool, publish: Option<&KansoPublish>) -> Phase {
    match (signed_in, publish) {
        (false, _) => Phase::SignedOut,
        (true, None) => Phase::Unpublished,
        (true, Some(_)) => Phase::Published,
    }
}

/// The expiry choice closest to how long a link was given. Pure.
pub fn expiry_choice(publish: &KansoPublish) -> Expiry {
    let Some(at) = publish.expires_at else {
        return Expiry::Never;
    };
    let given = at - publish.published_at;
    Expiry::ALL[1..]
        .iter()
        .copied()
        .min_by_key(|e| (e.at(0).unwrap_or(0) - given).abs())
        .unwrap_or(Expiry::Never)
}

/// Whether "Update published version" has something to do: the note
/// changed, another expiry was picked, or the link expired. Pure.
pub fn needs_update(
    publish: &KansoPublish,
    title: &str,
    body: &str,
    expiry: Expiry,
    now_ms: i64,
) -> bool {
    publish.is_outdated(title, body)
        || expiry != expiry_choice(publish)
        || publish.is_expired(now_ms)
}

/// "28 May at 09:41", from a local time in ms. Pure.
pub fn when_label(local_ms: i64) -> String {
    DateTime::from_timestamp_millis(local_ms).map_or_else(String::new, |at| {
        format!(
            "{} {} at {:02}:{:02}",
            at.day(),
            month(at.month()),
            at.hour(),
            at.minute()
        )
    })
}

fn month(m: u32) -> &'static str {
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][(m as usize).clamp(1, 12) - 1]
}

/// What an error means to the person publishing. Pure.
pub fn error_text(err: &ShareError) -> String {
    match err {
        ShareError::TooLarge(_) => {
            "This note is too large to publish: with its images it is over 10 MB.".into()
        }
        ShareError::Undecryptable => "Kanso couldn't seal the note. Try again.".into(),
        ShareError::Api(api) => match api {
            ApiError::Transport(_) => {
                "Couldn't reach your sync server. Check your connection and try again.".into()
            }
            ApiError::Unauthorized { .. } | ApiError::AccountDisabled => {
                "Your account signed this device out. Sign in again to publish.".into()
            }
            ApiError::QuotaExceeded { .. } => {
                "Your account has no room left for this. Unpublish another note, or remove some attachments.".into()
            }
            ApiError::RateLimited { .. } => {
                "Your sync server is busy. Try again in a minute.".into()
            }
            ApiError::NotFound | ApiError::MethodNotAllowed => {
                "Your sync server doesn't publish notes yet.".into()
            }
            other => format!("Couldn't publish the note ({other})."),
        },
    }
}

/// A task the sheet is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Busy {
    Publishing,
    Updating,
    Unpublishing,
}

pub struct PublishView {
    editor: Entity<NoteEditor>,
    store: Arc<Store>,
    source: Source,
    signed_in: bool,
    /// The server the link points at.
    server: String,
    expiry: Expiry,
    busy: Option<Busy>,
    error: Option<SharedString>,
    copied: bool,
    /// Unpublish was pressed once; the next press unpublishes.
    confirm_unpublish: bool,
}

impl PublishView {
    pub fn phase(&self) -> Phase {
        phase(self.signed_in, self.source.publish.as_ref())
    }

    pub fn is_busy(&self) -> bool {
        self.busy.is_some()
    }

    pub fn publish(&self) -> Option<&KansoPublish> {
        self.source.publish.as_ref()
    }

    #[cfg(test)]
    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    fn link(&self) -> Option<String> {
        self.publish().map(|p| p.link(&self.server))
    }

    fn outdated(&self, now_ms: i64) -> bool {
        self.publish().is_some_and(|p| {
            needs_update(
                p,
                &self.source.title,
                &self.source.body,
                self.expiry,
                now_ms,
            )
        })
    }

    fn set_expiry(&mut self, expiry: Expiry, cx: &mut Context<Self>) {
        self.expiry = expiry;
        cx.notify();
    }

    fn copy(&mut self, cx: &mut Context<Self>) {
        if let Some(link) = self.link() {
            cx.write_to_clipboard(ClipboardItem::new_string(link));
            self.copied = true;
            cx.notify();
        }
    }

    /// Publishes the note as typed now, or updates its link in place.
    pub fn run_publish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let services = cx.global::<Services>();
        let (Some(session), Some(api)) = (services.session(), services.shares.clone()) else {
            self.error = Some("Publishing needs your Kanso account. Sign in, then publish.".into());
            cx.notify();
            return;
        };
        // Saved first, so the page is the note as typed and its revision is real.
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let id = self.source.id.clone();
        match self.editor.read(cx).publish_source(cx) {
            Some(source) if source.id == id => self.source = source,
            _ => {
                self.error = Some("The note was closed. Open it again to publish it.".into());
                cx.notify();
                return;
            }
        }
        let palette = crate::export_html::print_palette(crate::preview_rich::palette(cx));
        let style = crate::export_style::PageStyle::from_cx(cx);
        let now = Clock::now_ms(cx);
        let expires_at = self.expiry.at(now);
        let existing = self.source.publish.clone();
        let store = self.store.clone();
        let Source {
            title, body, rev, ..
        } = self.source.clone();
        self.busy = Some(if existing.is_some() {
            Busy::Updating
        } else {
            Busy::Publishing
        });
        self.error = None;
        self.confirm_unpublish = false;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let html = crate::export_html::page(&store, &title, &body, &palette, &style);
                    let page = Page {
                        html: html.as_bytes(),
                        title: &title,
                        body: &body,
                        rev: &rev,
                    };
                    share::publish(&*api, &session, existing.as_ref(), &page, expires_at, now)
                })
                .await;
            this.update(cx, |this, cx| this.finish(result.map(Some), cx))
                .ok();
        })
        .detach();
    }

    /// Unpublishes the link: the first press asks, the second does it.
    pub fn run_unpublish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        if !self.confirm_unpublish {
            self.confirm_unpublish = true;
            cx.notify();
            return;
        }
        let Some(publish) = self.source.publish.clone() else {
            return;
        };
        let services = cx.global::<Services>();
        let (Some(session), Some(api)) = (services.session(), services.shares.clone()) else {
            self.error = Some("Unpublishing needs your Kanso account. Sign in first.".into());
            cx.notify();
            return;
        };
        self.busy = Some(Busy::Unpublishing);
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { share::unpublish(&*api, &session, &publish) })
                .await;
            this.update(cx, |this, cx| this.finish(result.map(|()| None), cx))
                .ok();
        })
        .detach();
    }

    /// Keeps what the server now holds in the note.
    fn finish(&mut self, result: Result<Option<KansoPublish>, ShareError>, cx: &mut Context<Self>) {
        self.busy = None;
        self.confirm_unpublish = false;
        match result {
            Ok(publish) => {
                let id = self.source.id.clone();
                let kept = self.editor.update(cx, |editor, cx| {
                    editor.set_publish(&id, publish.clone(), cx)
                });
                if !kept && let Err(err) = store_publish(&self.store, &id, publish.as_ref()) {
                    self.error = Some(format!("Couldn't save the link in the note: {err}").into());
                }
                if let Some(p) = &publish {
                    self.expiry = expiry_choice(p);
                }
                self.source.publish = publish;
                self.copied = false;
            }
            Err(err) => self.error = Some(error_text(&err).into()),
        }
        cx.notify();
    }
}

/// Keeps `publish` in a note that isn't open any more.
fn store_publish(store: &Store, id: &str, publish: Option<&KansoPublish>) -> anyhow::Result<()> {
    let Some(mut note) = store.note(id)? else {
        return Ok(());
    };
    KansoPublish::store(publish, &mut note.extra);
    store.put(
        DocType::Note,
        serde_json::to_value(&note)?,
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(())
}

pub(crate) fn paragraph(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(13.5))
        .line_height(px(20.))
        .text_color(colors.text_secondary)
        .child(text.into())
}

// no-prop-test: drawing; the publish snapshots show every phase in both
// themes, and `phase`, `needs_update` and `expiry_choice` carry the logic.
impl Render for PublishView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let now = Clock::now_ms(cx);
        let body = v_flex()
            .gap(px(14.))
            .debug_selector(|| "publish-sheet".into());
        let body = match self.phase() {
            Phase::SignedOut => body
                .child(paragraph(
                    "Publishing puts this note on the web at a link you can share. Anyone with the link can read it; your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(paragraph(
                    "It needs a Kanso account, so the copy has somewhere to live. Sign in, or create an account, then publish.",
                    colors,
                )),
            Phase::Unpublished => body
                .child(paragraph(
                    "Anyone with the link can read this note. The key that opens it is in the link itself, so your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(self.expiry_row(colors, cx))
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_muted)
                        .child("The link shows the note as it is now. Later edits stay on your devices until you update it. The note's title is sent unencrypted, so link previews can show it."),
                ),
            Phase::Published => {
                let publish = self.publish().cloned().expect("published");
                let link = self.link().unwrap_or_default();
                let mono = cx.theme().mono_font_family.clone();
                let published = when_label(Clock::local_ms(cx, publish.published_at));
                let expires = match publish.expires_at {
                    None => "The link doesn't expire.".to_owned(),
                    Some(at) if at <= now => format!("The link expired on {}.", when_label(Clock::local_ms(cx, at))),
                    Some(at) => format!("The link stops working on {}.", when_label(Clock::local_ms(cx, at))),
                };
                let changed = publish.is_outdated(&self.source.title, &self.source.body);
                body.child(paragraph(
                    "Anyone with this link can read the note. Your sync server keeps only an encrypted copy it can't read.",
                    colors,
                ))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .id("publish-link")
                                .debug_selector(|| "publish-link".into())
                                .flex_1()
                                .min_w_0()
                                .px(px(10.))
                                .py(px(7.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(colors.divider)
                                .bg(colors.list)
                                .font_family(mono)
                                .text_size(px(12.))
                                .text_color(colors.text)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(link),
                        )
                        .child(
                            Button::new("publish-copy")
                                .outline()
                                .icon(if self.copied { Lucide::Check } else { Lucide::Copy })
                                .label(if self.copied { "Copied" } else { "Copy" })
                                .debug_selector(|| "publish-copy".into())
                                .on_click(cx.listener(|this, _, _, cx| this.copy(cx))),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_muted)
                        .child(format!("Published {published}. {expires}")),
                )
                .when(changed, |el| {
                    el.child(
                        div()
                            .px(px(12.))
                            .py(px(8.))
                            .rounded(px(6.))
                            .bg(Hsla::from(colors.warning).opacity(0.14))
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .debug_selector(|| "publish-outdated".into())
                            .child("This note has changed since you published it. The link still shows the earlier version."),
                    )
                })
                .child(self.expiry_row(colors, cx))
                .when(self.confirm_unpublish, |el| {
                    el.child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.danger)
                            .child("Unpublishing stops the link working for everyone who has it. Publishing again makes a new link."),
                    )
                })
            }
        };
        body.when_some(self.error.clone(), |el, error| {
            el.child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.danger)
                    .debug_selector(|| "publish-error".into())
                    .child(error),
            )
        })
    }
}

impl PublishView {
    fn expiry_row(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        expiry_row(
            self.expiry,
            self.busy.is_some(),
            colors,
            cx,
            PublishView::set_expiry,
        )
    }
}

/// The "Link works for" row, for a note's sheet and a notebook's: `pick`
/// keeps the choice.
pub(crate) fn expiry_row<V: 'static>(
    current: Expiry,
    disabled: bool,
    colors: &Colors,
    cx: &mut Context<V>,
    pick: fn(&mut V, Expiry, &mut Context<V>),
) -> impl IntoElement {
    h_flex()
        .gap_3()
        .items_center()
        .child(
            div()
                .text_size(px(13.))
                .text_color(colors.text)
                .child("Link works for"),
        )
        .child(
            ButtonGroup::new("publish-expiry")
                .outline()
                .small()
                .disabled(disabled)
                .children(Expiry::ALL.iter().enumerate().map(|(ix, option)| {
                    let selected = *option == current;
                    Button::new(("publish-expiry", ix))
                        .selected(selected)
                        .debug_selector(move || format!("publish-expiry-{ix}"))
                        .child(
                            div()
                                .px_1()
                                .text_size(px(12.5))
                                .when(selected, |el| el.font_medium())
                                .text_color(if selected {
                                    colors.text
                                } else {
                                    colors.text_secondary
                                })
                                .child(if *option == Expiry::Never {
                                    "Always"
                                } else {
                                    option.label()
                                }),
                        )
                }))
                .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                    if let Some(&ix) = clicked.first() {
                        pick(this, Expiry::ALL[ix], cx);
                    }
                })),
        )
}

/// The sheet's buttons, which follow its phase and task.
pub struct PublishFooter {
    view: Entity<PublishView>,
    _observing: Subscription,
}

// no-prop-test: drawing; the publish snapshots show each phase's buttons.
impl Render for PublishFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = self.view.read(cx);
        let busy = view.busy;
        let now = Clock::now_ms(cx);
        let close = Button::new("publish-close")
            .label(if view.phase() == Phase::Published {
                "Done"
            } else {
                "Cancel"
            })
            .disabled(busy.is_some())
            .debug_selector(|| "publish-close".into())
            .on_click(|_, window, cx| window.close_dialog(cx));
        let target = self.view.clone();
        let row = h_flex().w_full().gap_2().items_center();
        match view.phase() {
            Phase::SignedOut => row
                .child(div().flex_1())
                .child(
                    Button::new("publish-create-account")
                        .ghost()
                        .label("Create account…")
                        .debug_selector(|| "publish-create-account".into())
                        .on_click(|_, window, cx| {
                            window.close_dialog(cx);
                            window
                                .dispatch_action(Box::new(crate::workspace::ShowCreateAccount), cx);
                        }),
                )
                .child(close)
                .child(
                    Button::new("publish-sign-in")
                        .primary()
                        .label("Sign in…")
                        .debug_selector(|| "publish-sign-in".into())
                        .on_click(|_, window, cx| {
                            window.close_dialog(cx);
                            window.dispatch_action(Box::new(crate::workspace::ShowSignIn), cx);
                        }),
                ),
            Phase::Unpublished => {
                let running = busy == Some(Busy::Publishing);
                row.child(div().flex_1()).child(close).child(
                    Button::new("publish-go")
                        .primary()
                        .label(if running { "Publishing…" } else { "Publish" })
                        .when(running, |b| b.icon(Lucide::RefreshCcw))
                        .loading(running)
                        .debug_selector(|| "publish-go".into())
                        .on_click(move |_, window, cx| {
                            target.update(cx, |view, cx| view.run_publish(window, cx));
                        }),
                )
            }
            Phase::Published => {
                let unpublishing = busy == Some(Busy::Unpublishing);
                let updating = busy == Some(Busy::Updating);
                let confirm = view.confirm_unpublish;
                let outdated = view.outdated(now);
                let unpublish_target = self.view.clone();
                row.child(
                    Button::new("publish-unpublish")
                        .ghost()
                        .when(confirm || unpublishing, |b| b.danger())
                        .label(if unpublishing {
                            "Unpublishing…"
                        } else if confirm {
                            "Unpublish link"
                        } else {
                            "Unpublish"
                        })
                        .loading(unpublishing)
                        .disabled(updating)
                        .debug_selector(|| "publish-unpublish".into())
                        .on_click(move |_, window, cx| {
                            unpublish_target.update(cx, |view, cx| view.run_unpublish(window, cx));
                        }),
                )
                .child(div().flex_1())
                .child(close)
                .when(outdated || updating, |row| {
                    row.child(
                        Button::new("publish-update")
                            .primary()
                            .label(if updating {
                                "Updating…"
                            } else {
                                "Update published version"
                            })
                            .when(updating, |b| b.icon(Lucide::RefreshCcw))
                            .loading(updating)
                            .disabled(unpublishing)
                            .debug_selector(|| "publish-update".into())
                            .on_click(move |_, window, cx| {
                                target.update(cx, |view, cx| view.run_publish(window, cx));
                            }),
                    )
                })
            }
        }
    }
}

/// Opens the Publish sheet for the note in `editor`. Returns the sheet, for
/// tests and snapshots to drive.
pub fn open_publish(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Option<Entity<PublishView>> {
    // A locked note's text stays sealed: publishing it would put it at a
    // link instead. So does the text of a note in a locked notebook.
    if editor.read(cx).lock_view() != crate::locked::LockView::Plain {
        let store = cx.global::<AppState>().store.clone();
        let in_locked_notebook = editor
            .read(cx)
            .note_id()
            .is_some_and(|id| store.note_in_locked_book(id).unwrap_or(false));
        window.push_notification(
            gpui_kit::component::notification::Notification::info(if in_locked_notebook {
                "Notes in a locked notebook can't be published. Move the note out and remove its lock first."
            } else {
                "Locked notes can't be published. Remove the lock first."
            }),
            cx,
        );
        return None;
    }
    let source = editor.read(cx).publish_source(cx)?;
    let services = cx.global::<Services>();
    let signed_in = services.signed_in();
    let server = services
        .session()
        .map(|s| s.server.clone())
        .unwrap_or_else(|| kanso_sync::DEFAULT_SERVER.to_owned());
    let store = cx.global::<AppState>().store.clone();
    let expiry = source.publish.as_ref().map_or(Expiry::Never, expiry_choice);
    let view = cx.new(|_| PublishView {
        editor,
        store,
        source,
        signed_in,
        server,
        expiry,
        busy: None,
        error: None,
        copied: false,
        confirm_unpublish: false,
    });
    let footer = cx.new(|cx| PublishFooter {
        view: view.clone(),
        _observing: cx.observe(&view, |_, _, cx| cx.notify()),
    });
    let dialog_view = view.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = dialog_view.clone();
        alert
            .title("Publish to the web")
            .width(px(SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(footer.clone())
            // Escape closes, but not while a task runs.
            .on_cancel(move |_, _, cx| !running.read(cx).is_busy())
    });
    Some(view)
}

#[cfg(test)]
mod tests {
    use super::{
        ApiError, Expiry, KansoPublish, Phase, ShareError, error_text, expiry_choice, needs_update,
        phase, share, when_label,
    };
    use proptest::prelude::*;
    use serde_json::Map;

    fn published(at: i64, expires: Option<i64>, title: &str, body: &str) -> KansoPublish {
        KansoPublish {
            share_id: share::new_share_id(),
            key: share::ShareKey::generate().to_text(),
            published_at: at,
            published_rev: "1-a".into(),
            expires_at: expires,
            content_hash: Some(share::content_hash(title, body)),
            short_url: None,
            other: Map::new(),
        }
    }

    #[test]
    fn examples() {
        let p = published(0, None, "T", "B");
        assert_eq!(phase(false, Some(&p)), Phase::SignedOut);
        assert_eq!(phase(true, None), Phase::Unpublished);
        assert_eq!(phase(true, Some(&p)), Phase::Published);
        assert!(!needs_update(&p, "T", "B", Expiry::Never, 5));
        assert!(needs_update(&p, "T", "B!", Expiry::Never, 5));
        assert!(needs_update(&p, "T", "B", Expiry::Week, 5));
        let week = published(0, Expiry::Week.at(0), "T", "B");
        assert_eq!(expiry_choice(&week), Expiry::Week);
        assert!(needs_update(
            &week,
            "T",
            "B",
            Expiry::Week,
            Expiry::Week.at(0).unwrap()
        ));
        assert_eq!(when_label(1_779_954_300_000 + 7_200_000), "28 May at 09:45");
        assert!(
            error_text(&ShareError::Api(ApiError::Transport("x".into()))).contains("connection")
        );
        assert!(error_text(&ShareError::TooLarge(1)).contains("10 MB"));
    }

    mod flow {
        use std::sync::Arc;

        use gpui_kit::component::WindowExt as _;
        use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};
        use kanso_sync::share::{KansoPublish, MemoryShares, PUBLISH_KEY, open_share};
        use kanso_sync::{DeviceIdentity, MemorySecrets, MemoryServer, Password};
        use serde_json::json;

        use super::super::{Phase, PublishView, open_publish};
        use crate::config::AppConfig;
        use crate::services::Services;
        use crate::workspace::{AppState, Workspace};

        struct Fixture {
            _dir: tempfile::TempDir,
            store: Arc<Store>,
            window: AnyWindowHandle,
            workspace: Entity<Workspace>,
            shares: Arc<MemoryShares>,
            note: String,
        }

        fn setup(cx: &mut TestAppContext, signed_in: bool) -> Fixture {
            cx.executor().allow_parking();
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            let book = store
                .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            let note = store
                .put(
                    DocType::Note,
                    json!({"title": "Plan", "body": "The secret plan.", "bookId": book, "share": "private"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            let shares = Arc::new(MemoryShares::new());
            let mut services = Services::default();
            services.shares = Some(shares.clone());
            if signed_in {
                let server = MemoryServer::new();
                let username = "alice";
                let session = kanso_sync::sign_up(
                    &server,
                    &MemorySecrets::new(),
                    kanso_sync::DEFAULT_SERVER,
                    username,
                    &Password::new("correct horse battery", username).unwrap(),
                    &DeviceIdentity::from_parts(&format!("l_{:032x}", 1), "Mac", [1; 32]),
                    &|_| {},
                )
                .unwrap()
                .session;
                services.set_signed_in(Arc::new(session));
            }
            let (data_dir, app_store) = (dir.path().to_path_buf(), store.clone());
            let (window, workspace) = cx.update(|cx| {
                gpui_kit::init(cx);
                cx.set_global(AppState {
                    store: app_store,
                    data_dir,
                    config: AppConfig::default(),
                });
                cx.set_global(services);
                gpui_kit::open_window(Default::default(), cx, |window, cx| {
                    cx.new(|cx| Workspace::new(window, cx))
                })
                .unwrap()
            });
            cx.run_until_parked();
            let library = cx.read(|cx| workspace.read(cx).library.clone());
            let id = note.clone();
            library.update(cx, |library, cx| library.select(Some(id), cx));
            cx.run_until_parked();
            Fixture {
                _dir: dir,
                store,
                window,
                workspace,
                shares,
                note,
            }
        }

        impl Fixture {
            fn open(&self, cx: &mut TestAppContext) -> Entity<PublishView> {
                let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
                self.window
                    .update(cx, |_, window, cx| open_publish(editor, window, cx))
                    .unwrap()
                    .expect("a note is open")
            }

            fn close(&self, cx: &mut TestAppContext) {
                self.window
                    .update(cx, |_, window, cx| window.close_dialog(cx))
                    .unwrap();
                cx.run_until_parked();
            }

            fn act(
                &self,
                cx: &mut TestAppContext,
                view: &Entity<PublishView>,
                f: impl FnOnce(
                    &mut PublishView,
                    &mut gpui_kit::Window,
                    &mut gpui_kit::Context<PublishView>,
                ),
            ) {
                self.window
                    .update(cx, |_, window, cx| {
                        view.update(cx, |v, cx| f(v, window, cx))
                    })
                    .unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                loop {
                    cx.run_until_parked();
                    if !cx.read(|cx| view.read(cx).is_busy()) {
                        break;
                    }
                    assert!(std::time::Instant::now() < deadline, "the task finished");
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }

            fn kept(&self) -> Option<KansoPublish> {
                let note = self.store.note(&self.note).unwrap().unwrap();
                assert_eq!(
                    note.extra.get("share"),
                    None,
                    "the share field belongs to the API's note shape"
                );
                KansoPublish::from_extra(&note.extra)
            }

            fn type_body(&self, cx: &mut TestAppContext, body: &str) {
                let editor = cx.read(|cx| self.workspace.read(cx).editor.clone());
                self.window
                    .update(cx, |_, window, cx| {
                        editor.update(cx, |editor, cx| editor.edit_for_test(body, window, cx))
                    })
                    .unwrap();
                cx.run_until_parked();
            }
        }

        // no-prop-test: an end-to-end flow in a window; the properties are
        // the pure functions' above and kanso-sync's share tests.
        #[gpui_kit::test]
        fn publish_update_and_unpublish(cx: &mut TestAppContext) {
            let f = setup(cx, true);
            let view = f.open(cx);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Unpublished);

            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            assert_eq!(cx.read(|cx| view.read(cx).error().cloned()), None);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Published);
            let first = f.kept().expect("the note keeps its link");
            let stored = f.shares.get(&first.share_id).unwrap();
            let page =
                open_share(&first.share_key().unwrap(), &first.share_id, &stored.sealed).unwrap();
            let page = String::from_utf8(page).unwrap();
            assert!(page.contains("<title>Plan</title>") && page.contains("The secret plan."));
            assert!(!page.contains("<script"));
            assert!(!String::from_utf8_lossy(&stored.sealed).contains("secret plan"));
            assert_eq!(stored.expires_at, None);
            f.close(cx);

            // Edited since: the sheet offers an update, which keeps the link.
            f.type_body(cx, "The revised plan.");
            let view = f.open(cx);
            let note = f.store.note(&f.note).unwrap().unwrap();
            assert!(first.is_outdated(&note.title, "The revised plan."));
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let second = f.kept().unwrap();
            assert_eq!(
                (&second.share_id, &second.key),
                (&first.share_id, &first.key)
            );
            assert!(!second.is_outdated("Plan", "The revised plan."));
            let stored = f.shares.get(&first.share_id).unwrap();
            let page =
                open_share(&first.share_key().unwrap(), &first.share_id, &stored.sealed).unwrap();
            assert!(
                String::from_utf8(page)
                    .unwrap()
                    .contains("The revised plan.")
            );

            // Unpublish asks first, then takes the link out of the note.
            f.act(cx, &view, |v, window, cx| v.run_unpublish(window, cx));
            assert!(
                f.shares.get(&first.share_id).is_some(),
                "the first press only asks"
            );
            f.act(cx, &view, |v, window, cx| v.run_unpublish(window, cx));
            assert!(f.shares.is_empty());
            assert_eq!(f.kept(), None);
            let note = f.store.note(&f.note).unwrap().unwrap();
            assert!(!note.extra.contains_key(PUBLISH_KEY));
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Unpublished);
        }

        // no-prop-test: one flow in a window; link building has its
        // properties in kanso-sync's share tests.
        #[gpui_kit::test]
        fn a_short_link_is_shown_and_copied_with_the_key(cx: &mut TestAppContext) {
            let f = setup(cx, true);
            f.shares.short_links(Some("https://rerout.test"));
            let view = f.open(cx);
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let kept = f.kept().expect("published");
            let short = kept.short_url.clone().expect("a short link");
            let stored = f.shares.get(&kept.share_id).unwrap();
            assert_eq!(stored.title, "Plan", "the title goes with the upload");
            assert_eq!(stored.short_url.as_deref(), Some(short.as_str()));
            f.act(cx, &view, |v, _, cx| v.copy(cx));
            let copied = cx.read_from_clipboard().and_then(|item| item.text());
            assert_eq!(copied, Some(format!("{short}#{}", kept.key)));
        }

        // no-prop-test: one flow in a window.
        #[gpui_kit::test]
        fn signed_out_it_explains_and_publishes_nothing(cx: &mut TestAppContext) {
            let f = setup(cx, false);
            let view = f.open(cx);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::SignedOut);
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            assert!(f.shares.is_empty());
            assert_eq!(f.kept(), None);
            assert!(cx.read(|cx| view.read(cx).error().is_some()));
        }

        // no-prop-test: one flow in a window.
        #[gpui_kit::test]
        fn a_failed_publish_says_why_and_keeps_nothing(cx: &mut TestAppContext) {
            let f = setup(cx, true);
            f.shares.go_offline(1);
            let view = f.open(cx);
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            let error = cx.read(|cx| view.read(cx).error().cloned()).unwrap();
            assert!(error.contains("connection"), "{error}");
            assert_eq!(f.kept(), None);
            assert_eq!(cx.read(|cx| view.read(cx).phase()), Phase::Unpublished);
            // Trying again works.
            f.act(cx, &view, |v, window, cx| v.run_publish(window, cx));
            assert!(f.kept().is_some());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round trip: a link published for any of the choices reads back
        /// as that choice, wherever in time it was published.
        #[test]
        fn prop_expiry_choice_round_trips(ix in 0usize..4, at in 0i64..4_000_000_000_000) {
            let expiry = Expiry::ALL[ix];
            let p = published(at, expiry.at(at), "t", "b");
            prop_assert_eq!(expiry_choice(&p), expiry);
        }

        /// A fresh publish never needs an update until the note, the choice
        /// or the clock moves past it.
        #[test]
        fn prop_fresh_publishes_are_up_to_date(ix in 0usize..4, title in "\\PC{0,20}", body in "\\PC{0,60}", at in 0i64..4_000_000_000_000) {
            let expiry = Expiry::ALL[ix];
            let p = published(at, expiry.at(at), &title, &body);
            prop_assert!(!needs_update(&p, &title, &body, expiry, at));
            let edited = format!("{body}.");
            prop_assert!(needs_update(&p, &title, &edited, expiry, at));
        }
    }
}
