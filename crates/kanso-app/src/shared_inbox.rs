//! The top of the Shared with me list: notes people shared with this
//! account that wait to be accepted, and comments that mention it, unread
//! (docs/ACCOUNTS.md › Collaboration). An invitation shows its owner's
//! fingerprint, verified or not, to compare before accepting. Accepting
//! brings the note in with the next sync; a mention opens its note.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::collab_model::Collab;
use crate::library::Library;
use crate::theme::Colors;
use crate::zoom::px;

/// What the badge beside an invitation's fingerprint says. Pure.
pub fn verified_label(verified: bool) -> &'static str {
    if verified { "Verified" } else { "Not verified" }
}

/// The line under an invitation's fingerprint. Pure.
pub fn compare_hint(owner: &str) -> String {
    format!("Compare it with what {owner} sees for you.")
}

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
            let owner = &invitation.owner;
            let verify = owner.account_id.clone();
            let badge = h_flex()
                .flex_none()
                .gap(px(4.))
                .px(px(6.))
                .h(px(18.))
                .rounded_full()
                .text_size(px(11.))
                .font_medium()
                .map(|el| {
                    if owner.verified {
                        el.bg(Hsla::from(colors.success).opacity(0.16))
                            .text_color(colors.success)
                            .child(Icon::new(Lucide::ShieldCheck).size(px(11.)))
                    } else {
                        el.bg(Hsla::from(colors.warning).opacity(0.16))
                            .text_color(colors.warning)
                            .child(Icon::new(Lucide::ShieldAlert).size(px(11.)))
                    }
                })
                .child(verified_label(owner.verified));
            // Two lines of three groups, the hint under them, then the
            // badge and the way to mark them verified.
            let fingerprint = v_flex()
                .gap(px(6.))
                .pl(px(25.))
                .child(
                    v_flex()
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .bg(colors.list)
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_size(px(12.5))
                        .debug_selector(move || format!("invitation-fingerprint-{ix}"))
                        .children(
                            crate::account_page::fingerprint_lines(&owner.fingerprint)
                                .into_iter()
                                .map(|line| div().child(line)),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(compare_hint(&owner.username)),
                )
                .child(
                    h_flex()
                        .gap(px(8.))
                        .child(badge)
                        .child(div().flex_1())
                        .when(!owner.verified, |el| {
                            el.child(
                                Button::new(("invitation-verify", ix))
                                    .outline()
                                    .xsmall()
                                    .label("Mark as verified")
                                    .debug_selector(move || format!("invitation-verify-{ix}"))
                                    .on_click(move |_, _, cx| {
                                        crate::collab::verify_owner(verify.clone(), cx)
                                    }),
                            )
                        }),
                );
            let top = h_flex()
                .gap(px(10.))
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
                );
            row(("invitation", ix))
                .flex_col()
                .items_stretch()
                .child(top)
                .child(fingerprint)
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

#[cfg(test)]
mod tests {
    use super::{compare_hint, verified_label};
    use proptest::prelude::*;

    #[test]
    fn an_invitation_says_whether_its_keys_were_compared() {
        assert_eq!(verified_label(true), "Verified");
        assert_eq!(verified_label(false), "Not verified");
        assert_eq!(
            compare_hint("user1"),
            "Compare it with what user1 sees for you."
        );
    }

    proptest! {
        /// The hint names the owner, as one sentence on one line.
        #[test]
        fn prop_the_hint_names_the_owner(owner in "[a-z][a-z0-9_-]{0,20}") {
            let hint = compare_hint(&owner);
            prop_assert!(hint.contains(&owner));
            prop_assert!(hint.ends_with('.') && !hint.contains('\n'));
        }
    }
}
