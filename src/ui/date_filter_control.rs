//! Preset dropdown plus a custom date range, shared by list screens.
use super::date_field;
use super::widgets::{choice, choice_value};
use chelete_lib::date_filter::{date_range, DateFilter};
use chrono::{Datelike, Duration, NaiveDate};
use gpui_kit::component::date_picker::{DatePicker, DatePickerState, DateRangePreset};
use gpui_kit::{div, prelude::*, rems, App, Context, Div, Entity, Subscription, Window};
use gpui_omarchy::{select, ChoiceState};

pub struct DateFilterControl {
    pub preset: Entity<ChoiceState>,
    pub range: Entity<DatePickerState>,
}

impl DateFilterControl {
    pub fn new<V: 'static>(window: &mut Window, cx: &mut Context<V>) -> Self {
        let items = DateFilter::ALL_FILTERS.iter().map(|f| (f.value().to_string(), f.label().to_string())).collect();
        Self {
            preset: choice(items, Some(DateFilter::All.value()), window, cx),
            range: date_field::range(window, cx),
        }
    }

    pub fn filter(&self, cx: &App) -> DateFilter {
        choice_value(&self.preset, cx)
            .and_then(|v| DateFilter::from_value(&v))
            .unwrap_or(DateFilter::All)
    }

    /// Inclusive ISO `(start, end)` for the current selection.
    pub fn bounds(&self, today: NaiveDate, cx: &App) -> (String, String) {
        let (start, end) = date_field::picked_range(&self.range, cx);
        date_range(self.filter(cx), start, end, today)
    }

    /// Re-render `owner` whenever the selection changes.
    pub fn observe<V: 'static>(&self, cx: &mut Context<V>, on_change: impl Fn(&mut V, &mut Context<V>) + Clone + 'static) -> Vec<Subscription> {
        let a = on_change.clone();
        vec![
            cx.observe(&self.preset, move |this, _, cx| a(this, cx)),
            cx.observe(&self.range, move |this, _, cx| on_change(this, cx)),
        ]
    }

    pub fn render(&self, today: NaiveDate, window: &mut Window, cx: &mut App) -> Div {
        let custom = self.filter(cx) == DateFilter::Custom;
        let presets = range_presets(today);
        div()
            .flex()
            .items_center()
            .gap(rems(0.5))
            .child(div().w(rems(13.)).child(select("date-filter", &self.preset, window, cx)))
            .when(custom, |row| {
                row.child(
                    div().w(rems(22.)).child(
                        DatePicker::new(&self.range)
                            .presets(presets)
                            .number_of_months(2)
                            .placeholder("Start — End")
                            .cleanable(true),
                    ),
                )
            })
    }
}

fn range_presets(today: NaiveDate) -> Vec<DateRangePreset> {
    let first_of_month = today.with_day(1).unwrap();
    let last_month_end = first_of_month - Duration::days(1);
    vec![
        DateRangePreset::range("Last 7 days", today - Duration::days(6), today),
        DateRangePreset::range("Last 30 days", today - Duration::days(29), today),
        DateRangePreset::range("This month", first_of_month, today),
        DateRangePreset::range("Last month", last_month_end.with_day(1).unwrap(), last_month_end),
        DateRangePreset::range("This year", NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap(), today),
    ]
}
