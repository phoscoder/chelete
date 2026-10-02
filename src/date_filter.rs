//! Date presets for list filters. Ranges are inclusive ISO date strings, so
//! they compare directly against `transaction_date`.
use chrono::{Datelike, Duration, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFilter {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    ThisYear,
    LastYear,
    All,
    Custom,
}

impl DateFilter {
    pub const ALL_FILTERS: [DateFilter; 10] = [
        DateFilter::Today,
        DateFilter::Yesterday,
        DateFilter::ThisWeek,
        DateFilter::LastWeek,
        DateFilter::ThisMonth,
        DateFilter::LastMonth,
        DateFilter::ThisYear,
        DateFilter::LastYear,
        DateFilter::All,
        DateFilter::Custom,
    ];

    pub fn value(self) -> &'static str {
        match self {
            DateFilter::Today => "today",
            DateFilter::Yesterday => "yesterday",
            DateFilter::ThisWeek => "this_week",
            DateFilter::LastWeek => "last_week",
            DateFilter::ThisMonth => "this_month",
            DateFilter::LastMonth => "last_month",
            DateFilter::ThisYear => "this_year",
            DateFilter::LastYear => "last_year",
            DateFilter::All => "all",
            DateFilter::Custom => "custom",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DateFilter::Today => "Today",
            DateFilter::Yesterday => "Yesterday",
            DateFilter::ThisWeek => "This Week",
            DateFilter::LastWeek => "Last Week",
            DateFilter::ThisMonth => "This Month",
            DateFilter::LastMonth => "Last Month",
            DateFilter::ThisYear => "This Year",
            DateFilter::LastYear => "Last Year",
            DateFilter::All => "All Time",
            DateFilter::Custom => "Custom Range",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL_FILTERS.into_iter().find(|f| f.value() == value)
    }
}

const MIN: &str = "0000-01-01";
const MAX: &str = "9999-12-31";

fn iso(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn start_of_week(d: NaiveDate) -> NaiveDate {
    d - Duration::days(i64::from(d.weekday().num_days_from_sunday()))
}

/// Inclusive `(start, end)` for a preset. Weeks start on Sunday. `custom`
/// uses the given bounds, leaving either side open when absent.
pub fn date_range(
    filter: DateFilter,
    custom_start: Option<NaiveDate>,
    custom_end: Option<NaiveDate>,
    today: NaiveDate,
) -> (String, String) {
    let first_of_month = today.with_day(1).unwrap();
    let last_month_end = first_of_month - Duration::days(1);
    let week_start = start_of_week(today);
    let last_week_end = week_start - Duration::days(1);
    match filter {
        DateFilter::Today => (iso(today), iso(today)),
        DateFilter::Yesterday => {
            let y = today - Duration::days(1);
            (iso(y), iso(y))
        }
        DateFilter::ThisWeek => (iso(week_start), iso(today)),
        DateFilter::LastWeek => (iso(last_week_end - Duration::days(6)), iso(last_week_end)),
        DateFilter::ThisMonth => (iso(first_of_month), iso(today)),
        DateFilter::LastMonth => (iso(last_month_end.with_day(1).unwrap()), iso(last_month_end)),
        DateFilter::ThisYear => (iso(NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap()), iso(today)),
        DateFilter::LastYear => (
            iso(NaiveDate::from_ymd_opt(today.year() - 1, 1, 1).unwrap()),
            iso(NaiveDate::from_ymd_opt(today.year() - 1, 12, 31).unwrap()),
        ),
        DateFilter::All => (MIN.into(), MAX.into()),
        DateFilter::Custom => (
            custom_start.map(iso).unwrap_or_else(|| MIN.into()),
            custom_end.map(iso).unwrap_or_else(|| MAX.into()),
        ),
    }
}

pub fn is_date_in_range(date: &str, start: &str, end: &str) -> bool {
    date >= start && date <= end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // Friday 2 October 2026.
    fn today() -> NaiveDate {
        d(2026, 10, 2)
    }

    fn range(f: DateFilter) -> (String, String) {
        date_range(f, None, None, today())
    }

    #[test]
    fn single_day_presets() {
        assert_eq!(range(DateFilter::Today), ("2026-10-02".into(), "2026-10-02".into()));
        assert_eq!(range(DateFilter::Yesterday), ("2026-10-01".into(), "2026-10-01".into()));
    }

    #[test]
    fn weeks_start_on_sunday() {
        assert_eq!(range(DateFilter::ThisWeek), ("2026-09-27".into(), "2026-10-02".into()));
        assert_eq!(range(DateFilter::LastWeek), ("2026-09-20".into(), "2026-09-26".into()));
    }

    #[test]
    fn this_week_on_a_sunday_is_just_that_day() {
        let sunday = d(2026, 10, 4);
        assert_eq!(date_range(DateFilter::ThisWeek, None, None, sunday), ("2026-10-04".into(), "2026-10-04".into()));
    }

    #[test]
    fn month_presets() {
        assert_eq!(range(DateFilter::ThisMonth), ("2026-10-01".into(), "2026-10-02".into()));
        assert_eq!(range(DateFilter::LastMonth), ("2026-09-01".into(), "2026-09-30".into()));
    }

    #[test]
    fn last_month_across_a_year_boundary() {
        assert_eq!(
            date_range(DateFilter::LastMonth, None, None, d(2026, 1, 15)),
            ("2025-12-01".into(), "2025-12-31".into())
        );
    }

    #[test]
    fn year_presets() {
        assert_eq!(range(DateFilter::ThisYear), ("2026-01-01".into(), "2026-10-02".into()));
        assert_eq!(range(DateFilter::LastYear), ("2025-01-01".into(), "2025-12-31".into()));
    }

    #[test]
    fn all_time_covers_everything() {
        let (s, e) = range(DateFilter::All);
        assert!(is_date_in_range("1999-05-05", &s, &e));
        assert!(is_date_in_range("2999-05-05", &s, &e));
    }

    #[test]
    fn custom_ranges_may_be_open_ended() {
        let c = |a, b| date_range(DateFilter::Custom, a, b, today());
        assert_eq!(c(Some(d(2026, 3, 1)), Some(d(2026, 3, 31))), ("2026-03-01".into(), "2026-03-31".into()));
        assert_eq!(c(Some(d(2026, 3, 1)), None).1, "9999-12-31");
        assert_eq!(c(None, Some(d(2026, 3, 31))).0, "0000-01-01");
    }

    #[test]
    fn range_membership_is_inclusive() {
        assert!(is_date_in_range("2026-03-01", "2026-03-01", "2026-03-31"));
        assert!(is_date_in_range("2026-03-31", "2026-03-01", "2026-03-31"));
        assert!(!is_date_in_range("2026-04-01", "2026-03-01", "2026-03-31"));
    }

    #[test]
    fn values_round_trip() {
        for f in DateFilter::ALL_FILTERS {
            assert_eq!(DateFilter::from_value(f.value()), Some(f));
        }
        assert_eq!(DateFilter::from_value("fortnight"), None);
    }
}
