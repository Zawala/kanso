//! Settings › General › Discovery: the switches for
//! the features that find notes by their words (`related.rs`, `ocr.rs`),
//! and where each stands.

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::Button;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, v_flex};
use gpui_kit::*;

use crate::ocr::{self, ImageText};
use crate::related::{self, Related, Status};
use crate::settings::{Row, SettingsView, section};
use crate::theme::Colors;
use crate::workspace::AppState;

/// What the device is called in the settings' words.
pub fn this_device() -> &'static str {
    "this device"
}

/// "12.4 MB".
pub fn megabytes(bytes: u64) -> String {
    format!("{:.1}\u{a0}MB", bytes as f64 / 1_000_000.)
}

/// The status row's title and line for related notes in `status`; none
/// while it is off.
pub fn related_words(status: &Status) -> Option<(String, String)> {
    let count = |n: usize| crate::text_stats::format_count(n);
    Some(match status {
        Status::Off => return None,
        Status::Downloading { done, total } => (
            format!(
                "Downloading the model… {}%",
                related::percent(*done, *total)
            ),
            format!("{} of {}", megabytes(*done), megabytes(*total)),
        ),
        Status::Loading => ("Getting ready…".to_owned(), "Checking the model".to_owned()),
        Status::Indexing { done, total } => (
            "Indexing notes…".to_owned(),
            format!("{} of {}", count(*done), count(*total)),
        ),
        Status::Ready { notes } => (
            "Ready".to_owned(),
            format!(
                "{} indexed. New and changed notes are indexed as you write.",
                crate::text_stats::plural(*notes, "note", "notes")
            ),
        ),
        Status::Failed(message) => ("Related notes stopped".to_owned(), message.clone()),
    })
}

/// The Discovery section, for the General page.
// no-prop-test: layout and switches; `related_words` says what it shows,
// and the settings snapshots show each state.
pub fn sections(cx: &mut Context<SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let discovery = cx.global::<AppState>().config.discovery;
    let status = cx
        .try_global::<Related>()
        .map(|r| r.status.clone())
        .unwrap_or_default();
    let device = this_device();
    let mut related_rows = vec![
        Row::new(Lucide::Sparkles, "Related notes")
            .detail(format!(
                "Lists notes about the same things as the open one, beside it. Turning this on downloads a small language model (about 30\u{a0}MB) once. It runs on {device}: nothing about your notes is sent anywhere."
            ))
            .trailing(
                div().debug_selector(|| "related-notes".into()).child(
                    Switch::new("related-notes")
                        .checked(discovery.related_notes)
                        .on_click(|on, _, cx| related::set_enabled(*on, cx)),
                ),
            ),
    ];
    if discovery.related_notes
        && let Some((title, detail)) = related_words(&status)
    {
        let icon = match status {
            Status::Downloading { .. } => Lucide::Download,
            Status::Failed(_) => Lucide::TriangleAlert,
            Status::Ready { .. } => Lucide::Check,
            _ => Lucide::LoaderCircle,
        };
        let mut row = Row::new(icon, title).detail(detail);
        if let Status::Downloading { done, total } = status {
            row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
        }
        if let Status::Indexing { done, total } = status {
            row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
        }
        if matches!(status, Status::Failed(_)) {
            row = row.trailing(
                Button::new("related-retry")
                    .outline()
                    .small()
                    .label("Try Again")
                    .on_click(|_, _, cx| related::start(cx)),
            );
        }
        related_rows.push(row);
    }
    let mut rows = related_rows;

    let image_row = if ocr::supported() {
        let reading = cx
            .try_global::<ImageText>()
            .map(|s| s.status.clone())
            .unwrap_or_default();
        let detail = match reading {
            ocr::Status::Reading { left } if discovery.image_text && left > 0 => format!(
                "Reading images… {} to go.",
                crate::text_stats::format_count(left)
            ),
            _ => format!(
                "Reads the words in pictures you attach, on {device}, so search finds the notes they're in. The words sync with the picture."
            ),
        };
        Row::new(Lucide::ScanText, "Search text in images")
            .detail(detail)
            .trailing(
                div().debug_selector(|| "image-text".into()).child(
                    Switch::new("image-text")
                        .checked(discovery.image_text)
                        .on_click(|on, _, cx| ocr::set_enabled(*on, cx)),
                ),
            )
    } else {
        Row::new(Lucide::ScanText, "Search text in images").detail(
            "Kanso reads the words in pictures on macOS. Words read there sync with the picture, and search finds them here too.",
        )
    };
    rows.push(image_row);
    vec![section("Discovery", rows, colors)]
}

/// A short bar `share` (0 to 1) of the way full.
// no-prop-test: drawing; the settings snapshots show it.
fn meter(share: f32, colors: &Colors) -> impl IntoElement {
    v_flex().w(px(96.)).child(
        div()
            .w_full()
            .h(px(4.))
            .rounded_full()
            .bg(colors.fill)
            .child(
                div()
                    .h_full()
                    .w(px(96. * share.clamp(0., 1.)))
                    .rounded_full()
                    .bg(colors.accent),
            ),
    )
}

#[cfg(test)]
mod tests {
    use super::{Status, related_words};
    use proptest::prelude::*;

    #[test]
    fn related_words_examples() {
        assert_eq!(related_words(&Status::Off), None);
        assert_eq!(
            related_words(&Status::Downloading {
                done: 12_400_000,
                total: 31_121_000
            }),
            Some((
                "Downloading the model… 39%".into(),
                "12.4\u{a0}MB of 31.1\u{a0}MB".into()
            ))
        );
        assert_eq!(
            related_words(&Status::Indexing {
                done: 120,
                total: 2_000
            })
            .unwrap()
            .1,
            "120 of 2,000"
        );
        assert!(
            related_words(&Status::Ready { notes: 1 })
                .unwrap()
                .1
                .starts_with("1 note indexed")
        );
    }

    proptest! {
        /// Every state but Off says something, and a failure says why.
        #[test]
        fn prop_every_state_reads(done in 0u64..50_000_000, total in 1u64..50_000_000, n in 0usize..100_000, why in "[a-z ]{1,30}") {
            for status in [
                Status::Downloading { done, total },
                Status::Loading,
                Status::Indexing { done: n / 2, total: n },
                Status::Ready { notes: n },
                Status::Failed(why.clone()),
            ] {
                let (title, detail) = related_words(&status).unwrap();
                prop_assert!(!title.is_empty() && !detail.is_empty());
                if let Status::Failed(_) = status {
                    prop_assert_eq!(&detail, &why);
                }
            }
        }
    }
}
