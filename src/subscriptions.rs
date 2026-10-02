//! Subscription scheduling and labels.
use crate::commands::Subscription;
use chrono::{Datelike, Duration, NaiveDate};

pub const FREQUENCIES: [(&str, &str); 6] = [
    ("weekly", "Weekly"),
    ("bi_weekly", "Bi-weekly"),
    ("monthly", "Monthly"),
    ("quarterly", "Quarterly"),
    ("bi_yearly", "Bi-yearly"),
    ("yearly", "Yearly"),
];

pub fn frequency_label(value: &str) -> String {
    FREQUENCIES
        .iter()
        .find(|(v, _)| *v == value)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| value.to_string())
}

/// Subscriptions with the given frequency, or all of them for `"all"`.
pub fn filter_by_frequency<'a>(subs: &'a [Subscription], frequency: &str) -> Vec<&'a Subscription> {
    subs.iter()
        .filter(|s| frequency == "all" || s.frequency == frequency)
        .collect()
}

pub fn total_amount(subs: &[&Subscription]) -> i64 {
    subs.iter().map(|s| s.amount).sum()
}

enum Interval {
    Days(i64),
    Months(u32),
}

fn interval(frequency: &str) -> Option<Interval> {
    Some(match frequency {
        "weekly" => Interval::Days(7),
        "bi_weekly" => Interval::Days(14),
        "monthly" => Interval::Months(1),
        "quarterly" => Interval::Months(3),
        "bi_yearly" => Interval::Months(6),
        "yearly" => Interval::Months(12),
        _ => return None,
    })
}

/// Add months the way JavaScript's `setFullYear(y, m + n, d)` does: a day past
/// the end of the target month spills into the following month.
fn add_months_overflowing(date: NaiveDate, months: u32) -> Option<NaiveDate> {
    let total = date.month0() + months;
    let year = date.year() + (total / 12) as i32;
    let first = NaiveDate::from_ymd_opt(year, total % 12 + 1, 1)?;
    first.checked_add_signed(Duration::days(i64::from(date.day()) - 1))
}

/// The first payment on or after `today`, stepping forward from `start`.
/// Mirrors the web UI: unknown frequencies return the start date unchanged,
/// and stepping stops after 120 iterations.
pub fn next_payment_date(start: Option<&str>, frequency: &str, today: NaiveDate) -> Option<NaiveDate> {
    let start = NaiveDate::parse_from_str(start?, "%Y-%m-%d").ok()?;
    let Some(step) = interval(frequency) else {
        return Some(start);
    };
    let mut next = start;
    let mut iterations = 0;
    while next < today && iterations < 120 {
        next = match step {
            Interval::Days(d) => next.checked_add_signed(Duration::days(d))?,
            Interval::Months(m) => add_months_overflowing(next, m)?,
        };
        iterations += 1;
    }
    Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn future_start_is_returned_as_is() {
        assert_eq!(next_payment_date(Some("2026-12-01"), "monthly", d(2026, 6, 1)), Some(d(2026, 12, 1)));
    }

    #[test]
    fn today_counts_as_due() {
        assert_eq!(next_payment_date(Some("2026-06-01"), "monthly", d(2026, 6, 1)), Some(d(2026, 6, 1)));
    }

    #[test]
    fn monthly_rolls_forward() {
        assert_eq!(next_payment_date(Some("2026-01-15"), "monthly", d(2026, 6, 20)), Some(d(2026, 7, 15)));
    }

    #[test]
    fn weekly_and_bi_weekly() {
        assert_eq!(next_payment_date(Some("2026-06-01"), "weekly", d(2026, 6, 10)), Some(d(2026, 6, 15)));
        assert_eq!(next_payment_date(Some("2026-06-01"), "bi_weekly", d(2026, 6, 10)), Some(d(2026, 6, 15)));
    }

    #[test]
    fn longer_periods() {
        assert_eq!(next_payment_date(Some("2025-01-10"), "quarterly", d(2026, 6, 1)), Some(d(2026, 7, 10)));
        assert_eq!(next_payment_date(Some("2025-01-10"), "bi_yearly", d(2026, 6, 1)), Some(d(2026, 7, 10)));
        assert_eq!(next_payment_date(Some("2024-03-05"), "yearly", d(2026, 6, 1)), Some(d(2027, 3, 5)));
    }

    #[test]
    fn month_end_spills_like_javascript() {
        // Jan 31 + 1 month is "Feb 31", which JavaScript normalises to Mar 3.
        assert_eq!(next_payment_date(Some("2026-01-31"), "monthly", d(2026, 2, 1)), Some(d(2026, 3, 3)));
    }

    #[test]
    fn missing_or_bad_start_has_no_next_payment() {
        assert_eq!(next_payment_date(None, "monthly", d(2026, 1, 1)), None);
        assert_eq!(next_payment_date(Some("not a date"), "monthly", d(2026, 1, 1)), None);
    }

    #[test]
    fn unknown_frequency_returns_the_start() {
        assert_eq!(next_payment_date(Some("2020-01-01"), "fortnightly-ish", d(2026, 1, 1)), Some(d(2020, 1, 1)));
    }

    #[test]
    fn stepping_stops_after_120_iterations() {
        // Weekly from 1990 would need ~1,900 steps; the cap leaves it in the past.
        let next = next_payment_date(Some("1990-01-01"), "weekly", d(2026, 1, 1)).unwrap();
        assert_eq!(next, d(1990, 1, 1) + Duration::days(7 * 120));
    }

    fn sub(freq: &str, amount: i64) -> Subscription {
        Subscription {
            id: freq.into(),
            name: freq.into(),
            amount,
            currency: "USD".into(),
            frequency: freq.into(),
            category_id: None,
            account_id: None,
            start_date: None,
            is_active: true,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn filters_and_totals() {
        let subs = vec![sub("monthly", 1500), sub("yearly", 12000), sub("monthly", 500)];
        let monthly = filter_by_frequency(&subs, "monthly");
        assert_eq!(monthly.len(), 2);
        assert_eq!(total_amount(&monthly), 2000);
        assert_eq!(filter_by_frequency(&subs, "all").len(), 3);
        assert_eq!(total_amount(&filter_by_frequency(&subs, "all")), 14000);
        assert!(filter_by_frequency(&subs, "weekly").is_empty());
    }

    #[test]
    fn labels() {
        assert_eq!(frequency_label("bi_weekly"), "Bi-weekly");
        assert_eq!(frequency_label("other"), "other");
    }
}
