//! The sheet that asks before an agent uses one of its own tools.
//!
//! An agent on this computer may ask, in the middle of an answer, to run a
//! tool of its own (read a file, run a command). Kanso can't see or limit
//! what that tool does, so it asks the person: the agent's name, the tool's
//! title and kind, and two buttons. **Reject** is the default: Enter,
//! Escape and clicking away all reject. **Allow once** allows this one
//! request; Kanso never answers "always".
//!
//! Requests arrive from an agent's reader thread and wait their turn here:
//! one sheet at a time, in order. A request with no window to show it in is
//! rejected at once, and one that timed out or was cancelled while it waited
//! is skipped, or closed if its sheet is up.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_acp::{PermissionPolicy, PermissionReply, PermissionRequest};

use crate::dialogs::FORM_WIDTH;
use crate::theme::Colors;
use crate::zoom::px;

/// The line every sheet carries.
pub const NO_SANDBOX: &str = "Kanso can't see or limit what the agent's own tools do.";

/// The longest tool title shown; the agent chooses the title, so it is cut.
const TITLE_MAX: usize = 240;

/// How often a sheet looks for its request having ended behind it.
const WATCH: Duration = Duration::from_millis(250);

/// What the person answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Reject,
    AllowOnce,
}

/// Everything a person can do to the sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    ClickReject,
    ClickAllowOnce,
    Enter,
    Escape,
    /// Clicking outside, or the dialog closing some other way.
    Dismiss,
}

/// What an input answers. Only clicking "Allow once" allows; Reject is the
/// default for every other way out.
pub fn choice(input: Input) -> Choice {
    match input {
        Input::ClickAllowOnce => Choice::AllowOnce,
        Input::ClickReject | Input::Enter | Input::Escape | Input::Dismiss => Choice::Reject,
    }
}

/// The sheet's title.
pub fn title(request: &PermissionRequest) -> String {
    format!("{} wants to use a tool", request.agent)
}

/// The tool's title as shown: one line, cut when very long.
pub fn tool_line(request: &PermissionRequest) -> String {
    let flat: String = request
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if flat.chars().count() <= TITLE_MAX {
        return flat;
    }
    let cut: String = flat.chars().take(TITLE_MAX).collect();
    format!("{}…", cut.trim_end())
}

/// The tool's kind as a line, if the agent gave one.
pub fn kind_line(request: &PermissionRequest) -> Option<String> {
    let kind = request.kind.as_deref()?.trim();
    (!kind.is_empty()).then(|| format!("Kind: {kind}"))
}

/// A request waiting for the person.
struct Ask {
    request: PermissionRequest,
    reply: PermissionReply,
}

/// The policy that queues each request for the sheet. If the app is gone,
/// the request goes with it and is rejected.
fn policy(queue: async_channel::Sender<(PermissionRequest, PermissionReply)>) -> PermissionPolicy {
    Arc::new(move |request, reply| {
        queue.try_send((request, reply)).ok();
    })
}

/// Starts answering agents' permission requests with the sheet. Until then,
/// and with no window, every request is rejected.
pub fn install(cx: &mut App) {
    kanso_ai::agent::set_permission_decider(Some(start(cx)));
}

/// The policy that shows each request in turn in the main window.
pub(crate) fn start(cx: &mut App) -> PermissionPolicy {
    let (queue, waiting) = async_channel::unbounded();
    cx.spawn(async move |cx| {
        while let Ok((request, reply)) = waiting.recv().await {
            present(Ask { request, reply }, cx).await;
        }
    })
    .detach();
    policy(queue)
}

/// One open sheet's answer, given once.
struct Sheet {
    reply: RefCell<Option<PermissionReply>>,
    done: async_channel::Sender<()>,
}

impl Sheet {
    /// Answers the request, unless something already did.
    fn answer(&self, input: Input) {
        let Some(reply) = self.reply.borrow_mut().take() else {
            return;
        };
        match choice(input) {
            Choice::AllowOnce => reply.allow_once(),
            Choice::Reject => reply.reject(),
        }
        self.done.try_send(()).ok();
    }

    /// The request ended without the person (time ran out, or the turn was
    /// cancelled), or was answered.
    fn is_over(&self) -> bool {
        self.reply
            .borrow()
            .as_ref()
            .is_none_or(PermissionReply::is_settled)
    }
}

/// Shows `ask` and returns once it is answered or over.
async fn present(ask: Ask, cx: &mut AsyncApp) {
    if ask.reply.is_settled() {
        return;
    }
    let (done, finished) = async_channel::bounded(1);
    let sheet = Rc::new(Sheet {
        reply: RefCell::new(Some(ask.reply)),
        done,
    });
    let request = ask.request;
    let shown = cx.update(|cx| {
        let Some((window, _)) = crate::workspace::main_workspace(cx) else {
            return false;
        };
        let sheet = sheet.clone();
        window
            .update(cx, move |_, window, cx| {
                open_sheet(&request, sheet, window, cx)
            })
            .is_ok()
    });
    if !shown {
        // No window to ask in: reject.
        sheet.answer(Input::Dismiss);
        return;
    }
    let watched = sheet.clone();
    let watch = cx.spawn(async move |cx| {
        loop {
            cx.background_executor().timer(WATCH).await;
            if watched.reply.borrow().is_none() {
                return;
            }
            if watched.is_over() {
                cx.update(close_sheet);
                watched.answer(Input::Dismiss);
                return;
            }
        }
    });
    finished.recv().await.ok();
    drop(watch);
}

/// Closes the sheet: its request ended behind it.
fn close_sheet(cx: &mut App) {
    if let Some((window, _)) = crate::workspace::main_workspace(cx) {
        window
            .update(cx, |_, window, cx| window.close_dialog(cx))
            .ok();
    }
}

fn open_sheet(request: &PermissionRequest, sheet: Rc<Sheet>, window: &mut Window, cx: &mut App) {
    let heading = title(request);
    let description = format!(
        "{} is asking before it uses one of its own tools.",
        request.agent
    );
    let tool = tool_line(request);
    let kind = kind_line(request);
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let reject = sheet.clone();
        let allow = sheet.clone();
        let enter = sheet.clone();
        let escape = sheet.clone();
        alert
            .title(heading.clone())
            .description(description.clone())
            .width(px(FORM_WIDTH))
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .debug_selector(|| "permission-tool".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.text)
                            .child(tool.clone()),
                    )
                    .children(kind.clone().map(|kind| {
                        div()
                            .debug_selector(|| "permission-kind".into())
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(kind)
                    }))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(17.))
                            .text_color(colors.text_secondary)
                            .child(NO_SANDBOX),
                    ),
            )
            .footer(
                h_flex()
                    .w_full()
                    .justify_end()
                    .gap(px(8.))
                    .child(
                        Button::new("permission-allow")
                            .label("Allow once")
                            .debug_selector(|| "permission-allow".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                allow.answer(Input::ClickAllowOnce);
                            }),
                    )
                    .child(
                        Button::new("permission-reject")
                            .label("Reject")
                            .primary()
                            .debug_selector(|| "permission-reject".into())
                            .on_click(move |_, window, cx| {
                                window.close_dialog(cx);
                                reject.answer(Input::ClickReject);
                            }),
                    ),
            )
            // Enter and Escape both reject.
            .on_ok(move |_, _, _| {
                enter.answer(Input::Enter);
                true
            })
            .on_cancel(move |_, _, _| {
                escape.answer(Input::Escape);
                true
            })
    });
}

#[cfg(test)]
mod tests {
    use super::{Choice, Input, choice, kind_line, title, tool_line};
    use kanso_acp::PermissionRequest;
    use proptest::prelude::*;

    fn request(title: &str, kind: Option<&str>) -> PermissionRequest {
        PermissionRequest {
            agent: "Claude Agent".to_owned(),
            title: title.to_owned(),
            kind: kind.map(str::to_owned),
        }
    }

    #[test]
    fn reject_is_the_default_and_only_allow_once_allows() {
        assert_eq!(choice(Input::Enter), Choice::Reject);
        assert_eq!(choice(Input::Escape), Choice::Reject);
        assert_eq!(choice(Input::Dismiss), Choice::Reject);
        assert_eq!(choice(Input::ClickReject), Choice::Reject);
        assert_eq!(choice(Input::ClickAllowOnce), Choice::AllowOnce);
    }

    #[test]
    fn the_sheet_names_the_agent_the_tool_and_its_kind() {
        let asked = request("Read ~/.ssh/id_ed25519", Some("read"));
        assert_eq!(title(&asked), "Claude Agent wants to use a tool");
        assert_eq!(tool_line(&asked), "Read ~/.ssh/id_ed25519");
        assert_eq!(kind_line(&asked).as_deref(), Some("Kind: read"));
        assert_eq!(kind_line(&request("x", None)), None);
        assert_eq!(kind_line(&request("x", Some("  "))), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Whatever title an agent gives, it shows as one line of bounded
        /// length.
        #[test]
        fn prop_a_tool_title_is_one_bounded_line(title in "\\PC{0,600}") {
            let shown = tool_line(&request(&title, None));
            prop_assert!(!shown.contains('\n'));
            prop_assert!(shown.chars().count() <= super::TITLE_MAX + 1);
        }

        /// No input but clicking Allow once ever allows.
        #[test]
        fn prop_only_the_allow_button_allows(
            inputs in proptest::collection::vec(
                prop_oneof![
                    Just(Input::ClickReject),
                    Just(Input::ClickAllowOnce),
                    Just(Input::Enter),
                    Just(Input::Escape),
                    Just(Input::Dismiss),
                ],
                1..8,
            ),
        ) {
            for input in inputs {
                prop_assert_eq!(choice(input) == Choice::AllowOnce, input == Input::ClickAllowOnce);
            }
        }
    }
}
