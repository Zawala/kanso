//! The top of the Shared with me list: notes people shared with this
//! account that wait to be accepted, and comments that mention it, unread
//! (docs/ACCOUNTS.md › Collaboration). Accepting brings the note in with
//! the next sync; a mention opens its note.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::collab_model::Collab;
use crate::library::Library;
use crate::theme::Colors;

/// Invitations and unread mentions, or nothing when there are none.
// no-prop-test: drawing; `Collab::unread` and the sidebar badge carry the
// counting, and the Shared with me snapshot shows it.
pub fn render(library: &Entity<Library>, cx: &App) -> Option<AnyElement> {
    let collab = cx.try_global::<Collab>()?;
    let unread: Vec<_> = collab.notices.iter().filter(|n| !n.read).cloned().collect();
    if collab.invitations.is_empty() && unread.is_empty() {
        return None;
    }
    let colors = Colors::get(cx);
    let row = |id: (&'static str, usize)| {
        h_flex()
            .id(id)
            .gap(px(10.))
            .px(px(12.))
            .py(px(8.))
            .mx_2()
            .rounded(px(6.))
            .bg(colors.fill)
            .text_size(px(13.))
            .text_color(colors.text)
    };
    let invitations = collab
        .invitations
        .iter()
        .enumerate()
        .map(|(ix, invitation)| {
            let (accept, decline) = (invitation.share_id.clone(), invitation.share_id.clone());
            row(("invitation", ix))
                .child(
                    Icon::new(Lucide::Users)
                        .size(px(15.))
                        .text_color(colors.text_secondary),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(div().truncate().child(format!(
                            "{} shared a note with you",
                            invitation.owner.username
                        )))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(colors.text_muted)
                                .child(invitation.role.label()),
                        ),
                )
                .child(
                    Button::new(("invitation-decline", ix))
                        .ghost()
                        .xsmall()
                        .label("Decline")
                        .on_click(move |_, _, cx| crate::collab::decline(decline.clone(), cx)),
                )
                .child(
                    Button::new(("invitation-accept", ix))
                        .primary()
                        .xsmall()
                        .label("Accept")
                        .debug_selector(move || format!("invitation-accept-{ix}"))
                        .on_click(move |_, _, cx| crate::collab::accept(accept.clone(), cx)),
                )
        });
    let mentions = unread.into_iter().enumerate().map(|(ix, notice)| {
        let library = library.clone();
        let title = if notice.title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            notice.title.clone()
        };
        row(("mention", ix))
            .cursor_pointer()
            .child(
                Icon::new(Lucide::AtSign)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .truncate()
                            .child(format!("{} mentioned you in {title}", notice.from_name)),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(notice.excerpt.clone()),
                    ),
            )
            .on_click(move |_, _, cx| crate::collab::open_notice(&notice, &library, cx))
    });
    let heading = |label: &'static str| {
        div()
            .px(px(20.))
            .pt(px(8.))
            .text_size(px(11.5))
            .font_semibold()
            .text_color(colors.text_secondary)
            .child(label)
    };
    Some(
        v_flex()
            .flex_none()
            .gap(px(6.))
            .pb(px(8.))
            .border_b_1()
            .border_color(colors.divider)
            .debug_selector(|| "shared-inbox".into())
            .when(!collab.invitations.is_empty(), |el| {
                el.child(heading("Invitations")).children(invitations)
            })
            .when(collab.unread() > 0, |el| {
                el.child(
                    h_flex()
                        .pr(px(12.))
                        .child(div().flex_1().child(heading("Mentions")))
                        .child(
                            Button::new("mentions-read")
                                .ghost()
                                .xsmall()
                                .label("Mark all read")
                                .on_click(|_, _, cx| crate::collab::mark_read(None, cx)),
                        ),
                )
                .children(mentions)
            })
            .into_any_element(),
    )
}
