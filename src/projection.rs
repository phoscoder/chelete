//! Balance projections: a straight line from a starting balance, income and
//! expense per period.
use chrono::{Duration, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodType {
    Weeks,
    Months,
    Years,
}

impl PeriodType {
    pub const ALL: [PeriodType; 3] = [PeriodType::Weeks, PeriodType::Months, PeriodType::Years];

    pub fn value(self) -> &'static str {
        match self {
            PeriodType::Weeks => "weeks",
            PeriodType::Months => "months",
            PeriodType::Years => "years",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.value() == value)
    }

    /// Singular: "week", "month", "year".
    pub fn unit(self) -> &'static str {
        match self {
            PeriodType::Weeks => "week",
            PeriodType::Months => "month",
            PeriodType::Years => "year",
        }
    }

    /// Plural: "weeks", "months", "years".
    pub fn plural(self) -> &'static str {
        self.value()
    }

    pub fn limit(self) -> u32 {
        match self {
            PeriodType::Weeks => 260,
            PeriodType::Months => 600,
            PeriodType::Years => 60,
        }
    }

    fn days(self) -> i64 {
        match self {
            PeriodType::Weeks => 7,
            PeriodType::Months => 30,
            PeriodType::Years => 365,
        }
    }

    fn per_year(self) -> f64 {
        match self {
            PeriodType::Weeks => 52.0,
            PeriodType::Months => 12.0,
            PeriodType::Years => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectionInput {
    pub starting_balance: i64,
    pub income: i64,
    pub expense: i64,
    pub period_type: PeriodType,
    pub periods: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionPoint {
    pub step: u32,
    pub label: String,
    pub balance: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionSummary {
    pub net_per_period: i64,
    pub final_balance: i64,
    pub total_increase: i64,
    /// Share of income kept, as a percentage. `None` without income.
    pub retention_rate: Option<f64>,
    pub projected_date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionResult {
    pub points: Vec<ProjectionPoint>,
    pub summary: ProjectionSummary,
    pub max_periods: u32,
}

pub fn period_label(period_type: PeriodType, step: u32) -> String {
    match step {
        0 => "Now".into(),
        1 => format!("1 {}", period_type.unit()),
        n => format!("{n} {}s", period_type.unit()),
    }
}

/// A subscription's amount expressed per projection period, in cents.
/// Unknown frequencies contribute nothing.
pub fn normalize_subscription_to_period(amount: i64, frequency: &str, period_type: PeriodType) -> i64 {
    let per_year = match frequency {
        "weekly" => 52.0,
        "bi_weekly" => 26.0,
        "monthly" => 12.0,
        "quarterly" => 4.0,
        "bi_yearly" => 2.0,
        "yearly" => 1.0,
        _ => return 0,
    };
    (amount as f64 * per_year / period_type.per_year()).round() as i64
}

pub fn compute_projection(input: ProjectionInput, today: NaiveDate) -> ProjectionResult {
    let ProjectionInput { starting_balance, income, expense, period_type, periods } = input;
    let net = income - expense;
    let points = (0..=periods)
        .map(|step| ProjectionPoint {
            step,
            label: period_label(period_type, step),
            balance: starting_balance + net * i64::from(step),
        })
        .collect();
    let final_balance = starting_balance + net * i64::from(periods);
    ProjectionResult {
        points,
        summary: ProjectionSummary {
            net_per_period: net,
            final_balance,
            total_increase: final_balance - starting_balance,
            retention_rate: (income > 0).then(|| net as f64 / income as f64 * 100.0),
            projected_date: today + Duration::days(period_type.days() * i64::from(periods)),
        },
        max_periods: period_type.limit(),
    }
}

/// Compact axis label: `$950`, `-$1.5k`, `$2.0M`.
pub fn format_axis(cents: i64) -> String {
    let dollars = cents.unsigned_abs() as f64 / 100.0;
    let sign = if cents < 0 { "-" } else { "" };
    if dollars >= 1_000_000.0 {
        format!("{sign}${:.1}M", dollars / 1_000_000.0)
    } else if dollars >= 1000.0 {
        format!("{sign}${:.1}k", dollars / 1000.0)
    } else {
        format!("{sign}${dollars:.0}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 2).unwrap()
    }

    fn input(income: i64, expense: i64, period_type: PeriodType, periods: u32) -> ProjectionInput {
        ProjectionInput { starting_balance: 0, income, expense, period_type, periods }
    }

    fn balances(r: &ProjectionResult) -> Vec<i64> {
        r.points.iter().map(|p| p.balance).collect()
    }

    #[test]
    fn canonical_example() {
        let r = compute_projection(input(120000, 10000, PeriodType::Months, 10), today());
        assert_eq!(r.summary.net_per_period, 110000);
        assert_eq!(r.summary.final_balance, 1100000);
        assert_eq!(r.summary.total_increase, 1100000);
        assert_eq!(r.points.len(), 11);
        assert_eq!(r.points[0].label, "Now");
        assert_eq!(r.points[0].balance, 0);
        assert_eq!(r.points[10].label, "10 months");
        assert_eq!(r.points[10].balance, 1100000);
    }

    #[test]
    fn balances_increase_linearly() {
        let r = compute_projection(input(300000, 120000, PeriodType::Weeks, 4), today());
        assert_eq!(balances(&r), vec![0, 180000, 360000, 540000, 720000]);
        let labels: Vec<_> = r.points.iter().map(|p| p.label.as_str()).collect();
        assert_eq!(labels, ["Now", "1 week", "2 weeks", "3 weeks", "4 weeks"]);
    }

    #[test]
    fn offsets_by_starting_balance() {
        let r = compute_projection(
            ProjectionInput { starting_balance: 500000, income: 100000, expense: 0, period_type: PeriodType::Months, periods: 3 },
            today(),
        );
        assert_eq!(r.summary.final_balance, 800000);
        assert_eq!(r.summary.total_increase, 300000);
        assert_eq!(r.points[0].balance, 500000);
        assert_eq!(r.points[3].balance, 800000);
    }

    #[test]
    fn flat_when_income_equals_expense() {
        let r = compute_projection(input(50000, 50000, PeriodType::Months, 5), today());
        assert_eq!(r.summary.net_per_period, 0);
        assert!(r.points.iter().all(|p| p.balance == 0));
    }

    #[test]
    fn declines_when_expenses_exceed_income() {
        let r = compute_projection(input(20000, 50000, PeriodType::Months, 4), today());
        assert_eq!(r.summary.net_per_period, -30000);
        assert_eq!(r.summary.final_balance, -120000);
        assert_eq!(balances(&r), vec![0, -30000, -60000, -90000, -120000]);
    }

    #[test]
    fn retention_rate() {
        let r = compute_projection(input(120000, 10000, PeriodType::Months, 10), today());
        assert!((r.summary.retention_rate.unwrap() - 91.6667).abs() < 0.001);
        let none = compute_projection(input(0, 10000, PeriodType::Months, 10), today());
        assert_eq!(none.summary.retention_rate, None);
    }

    #[test]
    fn years_period_type() {
        let r = compute_projection(input(1200000, 0, PeriodType::Years, 2), today());
        let labels: Vec<_> = r.points.iter().map(|p| p.label.as_str()).collect();
        assert_eq!(labels, ["Now", "1 year", "2 years"]);
        assert_eq!(r.summary.final_balance, 2400000);
    }

    #[test]
    fn respects_period_limits() {
        assert_eq!(PeriodType::Months.limit(), 600);
        let r = compute_projection(input(0, 0, PeriodType::Months, 600), today());
        assert_eq!(r.max_periods, 600);
        assert_eq!(r.points.len(), 601);
    }

    #[test]
    fn projected_date_is_periods_ahead() {
        let r = compute_projection(input(0, 0, PeriodType::Months, 10), today());
        assert_eq!(r.summary.projected_date, today() + Duration::days(300));
    }

    #[test]
    fn labels() {
        for p in PeriodType::ALL {
            assert_eq!(period_label(p, 0), "Now");
            assert_eq!(period_label(p, 1), format!("1 {}", p.unit()));
            assert_eq!(period_label(p, 3), format!("3 {}s", p.unit()));
        }
    }

    #[test]
    fn normalizes_subscriptions() {
        use PeriodType::*;
        let n = normalize_subscription_to_period;
        assert_eq!(n(10000, "monthly", Months), 10000);
        assert_eq!(n(12000, "monthly", Weeks), 2769);
        assert_eq!(n(10000, "monthly", Years), 120000);
        assert_eq!(n(1200000, "yearly", Months), 100000);
        assert_eq!(n(1000, "weekly", Months), 4333);
        assert_eq!(n(2000, "bi_weekly", Months), 4333);
        assert_eq!(n(30000, "quarterly", Months), 10000);
        assert_eq!(n(60000, "bi_yearly", Months), 10000);
        assert_eq!(n(500, "weekly", Weeks), 500);
        assert_eq!(n(1, "monthly", Weeks), 0);
        assert_eq!(n(10000, "daily", Months), 0);
    }

    #[test]
    fn axis_labels() {
        assert_eq!(format_axis(95000), "$950");
        assert_eq!(format_axis(-150000), "-$1.5k");
        assert_eq!(format_axis(200000000), "$2.0M");
        assert_eq!(format_axis(0), "$0");
    }

    #[test]
    fn period_type_round_trips() {
        for p in PeriodType::ALL {
            assert_eq!(PeriodType::from_value(p.value()), Some(p));
        }
        assert_eq!(PeriodType::from_value("decades"), None);
    }
}
