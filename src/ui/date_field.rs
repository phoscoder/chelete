//! Thin wrappers over gpui-kit's date picker so screens deal in `NaiveDate`.
use chrono::NaiveDate;
use gpui_kit::component::calendar::Date;
use gpui_kit::component::date_picker::DatePickerState;
use gpui_kit::{prelude::*, App, Context, Entity, Window};

pub const FORMAT: &str = "%Y-%m-%d";

pub fn single<V: 'static>(
    initial: Option<NaiveDate>,
    window: &mut Window,
    cx: &mut Context<V>,
) -> Entity<DatePickerState> {
    cx.new(|cx| {
        let mut state = DatePickerState::new(window, cx).date_format(FORMAT);
        if let Some(date) = initial {
            state.set_date(Date::Single(Some(date)), window, cx);
        }
        state
    })
}

pub fn range<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Entity<DatePickerState> {
    cx.new(|cx| DatePickerState::range(window, cx).date_format(FORMAT).number_of_months(2))
}

pub fn picked(state: &Entity<DatePickerState>, cx: &App) -> Option<NaiveDate> {
    match state.read(cx).date() {
        Date::Single(d) => d,
        Date::Range(start, _) => start,
    }
}

pub fn picked_range(state: &Entity<DatePickerState>, cx: &App) -> (Option<NaiveDate>, Option<NaiveDate>) {
    match state.read(cx).date() {
        Date::Range(a, b) => (a, b),
        Date::Single(d) => (d, d),
    }
}

pub fn parse_iso(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, FORMAT).ok()
}
