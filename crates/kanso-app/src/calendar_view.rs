//! The Journal's calendar sheet: a month, its days with a daily note
//! marked, today ringed. Clicking a day opens its note, made from the
//! daily template when it doesn't exist yet; ‹ and › go through the months.
//! The keyboard moves a day with the arrows, a month with Page Up and Page
//! Down, goes to today with T, and opens the day with Return.

use std::collections::HashSet;
use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::Date;
use kanso_store::{Origin, Store};

use crate::calendar::{MonthGrid, add_months, shift_month, system_week_start, weekday_labels};
use crate::journal::now;
use crate::theme::Colors;
use crate::workspace::Workspace;
use crate::zoom::px;

/// The sheet's width.
pub const SHEET_WIDTH: f32 = 392.;
const CELL: f32 = 44.;

actions!(kanso, [ShowCalendar]);
actions!(
    calendar_view,
    [
        PreviousDayCell,
        NextDayCell,
        PreviousWeek,
        NextWeek,
        PreviousMonth,
        NextMonth,
        GoToToday,
        OpenDay
    ]
);

pub fn bind_keys(cx: &mut App) {
    let context = Some("Calendar");
    cx.bind_keys([
        KeyBinding::new("left", PreviousDayCell, context),
        KeyBinding::new("right", NextDayCell, context),
        KeyBinding::new("up", PreviousWeek, context),
        KeyBinding::new("down", NextWeek, context),
        KeyBinding::new("pageup", PreviousMonth, context),
        KeyBinding::new("pagedown", NextMonth, context),
        KeyBinding::new("t", GoToToday, context),
        KeyBinding::new("enter", OpenDay, context),
    ]);
}

/// A day was picked: its note opens.
pub struct Picked(pub Date);

pub struct CalendarView {
    store: Arc<Store>,
    /// The month shown.
    pub year: i32,
    pub month: u32,
    /// The day the keyboard is on.
    pub focused: Date,
    pub today: Date,
    pub week_start: usize,
    /// The days that have a daily note.
    pub days: HashSet<Date>,
    focus: FocusHandle,
}

impl EventEmitter<Picked> for CalendarView {}

impl Focusable for CalendarView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl CalendarView {
    pub fn new(store: Arc<Store>, around: Date, today: Date, cx: &mut Context<Self>) -> Self {
        let mut view = CalendarView {
            store,
            year: around.year,
            month: around.month,
            focused: around,
            today,
            week_start: system_week_start(),
            days: HashSet::new(),
            focus: cx.focus_handle(),
        };
        view.read_days();
        view
    }

    /// Reads which days have a daily note.
    pub fn read_days(&mut self) {
        self.days = self
            .store
            .daily_dates()
            .unwrap_or_default()
            .into_iter()
            .collect();
    }

    fn focus_day(&mut self, day: Date, cx: &mut Context<Self>) {
        self.focused = day;
        self.year = day.year;
        self.month = day.month;
        cx.notify();
    }

    fn step_days(&mut self, days: i64, cx: &mut Context<Self>) {
        self.focus_day(self.focused.add_days(days), cx);
    }

    fn step_months(&mut self, months: i32, cx: &mut Context<Self>) {
        let (year, month) = shift_month(self.year, self.month, months);
        let focused = if (self.focused.year, self.focused.month) == (self.year, self.month) {
            add_months(self.focused, months)
        } else {
            Date::new(year, month, 1).unwrap_or(self.focused)
        };
        self.focused = focused;
        self.year = year;
        self.month = month;
        cx.notify();
    }

    fn pick(&mut self, day: Date, cx: &mut Context<Self>) {
        self.focused = day;
        cx.emit(Picked(day));
    }

    fn render_day(
        &self,
        day: Date,
        grid: &MonthGrid,
        focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let inside = grid.in_month(day);
        let has_note = self.days.contains(&day);
        let today = day == self.today;
        let selected = day == self.focused;
        let accent = Hsla::from(colors.accent);
        let label = if has_note {
            format!("{}, has a daily note", day.long_label())
        } else {
            day.long_label()
        };
        v_flex()
            .id(SharedString::from(format!("calendar-{}", day.iso())))
            .debug_selector(move || format!("calendar-{}", day.iso()))
            .w(px(CELL))
            .h(px(CELL))
            .items_center()
            .justify_center()
            .gap(px(2.))
            .rounded(px(8.))
            .cursor_pointer()
            .when(selected, |el| el.bg(colors.selected))
            .when(!selected, |el| el.hover(|el| el.bg(colors.hover)))
            .when(selected && focused, |el| {
                el.border_1().border_color(accent.opacity(0.5))
            })
            .child(
                div()
                    .size(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .when(today, |el| el.bg(colors.accent))
                    .text_size(px(13.5))
                    .when(today, |el| el.font_semibold())
                    .text_color(if today {
                        colors.button_text
                    } else if inside {
                        colors.text
                    } else {
                        colors.text_muted
                    })
                    .child(day.day.to_string()),
            )
            .child(div().size(px(5.)).rounded_full().when(has_note, |el| {
                el.bg(if inside { accent } else { accent.opacity(0.45) })
            }))
            .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
            .on_click(cx.listener(move |this, _, _, cx| this.pick(day, cx)))
            .into_any_element()
    }
}

// no-prop-test: drawing; the calendar snapshots show it, and `MonthGrid`
// carries the logic with its properties.
impl Render for CalendarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let grid = MonthGrid::new(self.year, self.month, self.week_start);
        let focused = self.focus.contains_focused(window, cx);
        let nav = |id: &'static str, icon: Lucide, tip: &'static str| {
            Button::new(id)
                .ghost()
                .small()
                .icon(icon)
                .tooltip(tip)
                .debug_selector(move || id.into())
        };
        let weeks: Vec<AnyElement> = grid
            .weeks
            .iter()
            .map(|week| {
                h_flex()
                    .gap(px(4.))
                    .children(
                        week.iter()
                            .map(|&day| self.render_day(day, &grid, focused, cx)),
                    )
                    .into_any_element()
            })
            .collect();
        v_flex()
            .key_context("Calendar")
            .track_focus(&self.focus)
            .debug_selector(|| "calendar-sheet".into())
            .gap(px(10.))
            .on_action(cx.listener(|this, _: &PreviousDayCell, _, cx| this.step_days(-1, cx)))
            .on_action(cx.listener(|this, _: &NextDayCell, _, cx| this.step_days(1, cx)))
            .on_action(cx.listener(|this, _: &PreviousWeek, _, cx| this.step_days(-7, cx)))
            .on_action(cx.listener(|this, _: &NextWeek, _, cx| this.step_days(7, cx)))
            .on_action(cx.listener(|this, _: &PreviousMonth, _, cx| this.step_months(-1, cx)))
            .on_action(cx.listener(|this, _: &NextMonth, _, cx| this.step_months(1, cx)))
            .on_action(cx.listener(|this, _: &GoToToday, _, cx| {
                let today = this.today;
                this.focus_day(today, cx)
            }))
            .on_action(cx.listener(|this, _: &OpenDay, _, cx| {
                let day = this.focused;
                this.pick(day, cx)
            }))
            .child(
                h_flex()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(15.))
                            .font_semibold()
                            .text_color(colors.text)
                            .debug_selector(|| "calendar-month".into())
                            .child(grid.title()),
                    )
                    .child(
                        nav("calendar-previous", Lucide::ChevronLeft, "Previous month")
                            .on_click(cx.listener(|this, _, _, cx| this.step_months(-1, cx))),
                    )
                    .child(
                        Button::new("calendar-today")
                            .ghost()
                            .small()
                            .label("Today")
                            .debug_selector(|| "calendar-today".into())
                            .on_click(cx.listener(|this, _, _, cx| {
                                let today = this.today;
                                this.focus_day(today, cx)
                            })),
                    )
                    .child(
                        nav("calendar-next", Lucide::ChevronRight, "Next month")
                            .on_click(cx.listener(|this, _, _, cx| this.step_months(1, cx))),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(4.))
                    .children(weekday_labels(self.week_start).map(|label| {
                        div()
                            .w(px(CELL))
                            .text_center()
                            .text_size(px(11.5))
                            .font_medium()
                            .text_color(colors.text_muted)
                            .child(label)
                    })),
            )
            .child(v_flex().gap(px(2.)).children(weeks))
            .child(
                div()
                    .pt(px(2.))
                    .text_size(px(12.))
                    .line_height(px(17.))
                    .text_color(colors.text_muted)
                    .child("A dot marks a day with a note. Click a day to open it, or to start it from the daily template."),
            )
    }
}

/// Opens the calendar on the open daily note's month, or today's. Returns
/// the sheet, for tests and snapshots to drive.
pub fn open_calendar(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) -> Entity<CalendarView> {
    let store = workspace.store.clone();
    let today = now(cx).date;
    let around = workspace
        .editor
        .read(cx)
        .note_id()
        .and_then(kanso_core::roles::daily_date)
        .unwrap_or(today);
    let view = cx.new(|cx| CalendarView::new(store, around, today, cx));
    workspace._calendar_events =
        Some(
            cx.subscribe_in(&view, window, |this, _, Picked(day), window, cx| {
                window.close_dialog(cx);
                this.open_day(*day, window, cx);
            }),
        );
    let dialog_view = view.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Journal")
            .width(px(SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(div())
    });
    window.focus(&view.focus_handle(cx), cx);
    view
}

impl Workspace {
    /// Opens day `day`'s note, made (from the daily template) when it
    /// doesn't exist yet, as `kanso-create-daily-note` makes it.
    pub(crate) fn open_day(&mut self, day: Date, window: &mut Window, cx: &mut Context<Self>) {
        let mut at = now(cx);
        if day == at.date {
            return self.open_today(window, cx);
        }
        at.date = day;
        self.flush(cx);
        match self.store.open_daily(at, Origin::Ui) {
            Ok(daily) => crate::journal::open_made(
                &self.library,
                &self.editor,
                daily.id,
                daily.cursor,
                false,
                window,
                cx,
            ),
            Err(err) => window.push_notification(
                gpui_kit::component::notification::Notification::error(format!(
                    "Couldn't open that day's note: {err}"
                )),
                cx,
            ),
        }
    }
}

pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowCalendar, window, cx| {
        this.close_settings(window, cx);
        open_calendar(this, window, cx);
    }))
}
