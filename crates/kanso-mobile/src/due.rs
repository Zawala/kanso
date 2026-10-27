//! Due dates in words: the chip a date gets in a list or on a card, and the
//! chips the preview draws.
//!
//! The words, the tone and the tooltip are `kanso_engine::tasks`, which the
//! desktop draws its chips from; `kanso-render` knows nothing of dates and
//! is handed a [`DueChips`] to ask.

use kanso_core::dates::DateTime;
use kanso_core::tasks::Due;
use kanso_engine::tasks::{self, Tone};
use kanso_render::html::{DueChip, DueChips, DueTone};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::journal::time_of;
use crate::library::Library;
use crate::model::{DueChipRow, DueTone as Tone16, LocalTime};

/// The chips of a preview drawn at local time `now`.
pub(crate) struct Chips {
    pub(crate) now: DateTime,
}

impl DueChips for Chips {
    fn chip(&self, url: &str) -> Option<DueChip> {
        let (due, checked) = tasks::parse_due_url(url)?;
        let chip = chip_row(due, checked, self.now);
        Some(DueChip {
            label: chip.label,
            tone: match chip.tone {
                Tone16::Plain => DueTone::Plain,
                Tone16::Today => DueTone::Today,
                Tone16::Overdue => DueTone::Overdue,
                Tone16::Done => DueTone::Done,
            },
            tooltip: chip.tooltip,
        })
    }
}

fn chip_row(due: Due, checked: bool, now: DateTime) -> DueChipRow {
    DueChipRow {
        label: tasks::chip_label(due, checked, now),
        tone: match tasks::tone(due, checked, now) {
            Tone::Plain => Tone16::Plain,
            Tone::Today => Tone16::Today,
            Tone::Overdue => Tone16::Overdue,
            Tone::Done => Tone16::Done,
        },
        tooltip: tasks::long_label(due),
    }
}

impl Library {
    /// The chip for a due date (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`, as a
    /// task row has it) of a task that is `checked` or not, at the phone's
    /// local time `now`. `None` when it isn't a date.
    pub fn due_chip(
        due: &str,
        checked: bool,
        now: LocalTime,
    ) -> Result<Option<DueChipRow>, KansoError> {
        guard(|| {
            let now = time_of(now)?;
            Ok(Due::parse(due).map(|due| chip_row(due, checked, now)))
        })
    }
}
