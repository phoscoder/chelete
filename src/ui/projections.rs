use super::widgets::*;
use chelete_lib::commands::{self, Subscription};
use chelete_lib::database::DbState;
use chelete_lib::format::{format_balance, format_money, parse_dollars_lenient};
use chelete_lib::projection::{
    compute_projection, format_axis, normalize_subscription_to_period, PeriodType, ProjectionInput,
    ProjectionResult,
};
use gpui_kit::base::input::InputState;
use gpui_kit::{div, prelude::*, rems, Context, Div, Entity, IntoElement, Render, SharedString, Subscription as GpuiSubscription, Window};
use gpui_omarchy::chart::LineChart;
use gpui_omarchy::{empty_state, input, panel, select, ActiveTheme as _, ChoiceState};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpenseSource {
    Manual,
    Subscriptions,
}

/// Everything the form collects, as typed.
pub struct FormValues<'a> {
    pub starting_balance: &'a str,
    pub income: &'a str,
    pub manual_expense: &'a str,
    pub source: ExpenseSource,
    pub subscriptions_expense: i64,
    pub period_type: PeriodType,
    pub periods: &'a str,
}

/// `None` until every field is valid, mirroring the web form.
pub fn build_input(v: &FormValues) -> Option<ProjectionInput> {
    let starting_balance = parse_dollars_lenient(v.starting_balance)?;
    let income = parse_dollars_lenient(v.income)?;
    let expense = match v.source {
        ExpenseSource::Subscriptions => v.subscriptions_expense,
        ExpenseSource::Manual => parse_dollars_lenient(v.manual_expense)?,
    };
    let periods: u32 = v.periods.trim().parse().ok()?;
    if periods < 1 || periods > v.period_type.limit() {
        return None;
    }
    Some(ProjectionInput { starting_balance, income, expense, period_type: v.period_type, periods })
}

pub struct ProjectionsScreen {
    db: Arc<DbState>,
    starting: Entity<InputState>,
    income: Entity<InputState>,
    expense: Entity<InputState>,
    periods: Entity<InputState>,
    source: Entity<ChoiceState>,
    period_type: Entity<ChoiceState>,
    subscriptions: Vec<Subscription>,
    _subs: Vec<GpuiSubscription>,
}

impl ProjectionsScreen {
    pub fn new(db: Arc<DbState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let starting = text_input_with("0.00", "0", window, cx);
        let income = text_input("e.g. 1200.00", window, cx);
        let expense = text_input("e.g. 100.00", window, cx);
        let periods = text_input_with("e.g. 10", "10", window, cx);
        let source = choice(
            vec![("manual".into(), "Manual".into()), ("subscriptions".into(), "Subscriptions".into())],
            Some("manual"),
            window,
            cx,
        );
        let period_type = choice(
            PeriodType::ALL.iter().map(|p| (p.value().to_string(), capitalize(p.plural()))).collect(),
            Some("months"),
            window,
            cx,
        );
        let mut subs = Vec::new();
        for input in [&starting, &income, &expense, &periods] {
            subs.push(cx.observe(input, |_, _, cx| cx.notify()));
        }
        for choice in [&source, &period_type] {
            subs.push(cx.observe(choice, |_, _, cx| cx.notify()));
        }
        let mut screen = Self { db, starting, income, expense, periods, source, period_type, subscriptions: Vec::new(), _subs: subs };
        screen.load();
        screen
    }

    fn load(&mut self) {
        self.subscriptions = commands::get_subscriptions(&self.db)
            .map(|all| all.into_iter().filter(|s| s.is_active).collect())
            .unwrap_or_default();
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load();
        cx.notify();
    }

    fn period(&self, cx: &gpui_kit::App) -> PeriodType {
        choice_value(&self.period_type, cx)
            .and_then(|v| PeriodType::from_value(&v))
            .unwrap_or(PeriodType::Months)
    }

    fn expense_source(&self, cx: &gpui_kit::App) -> ExpenseSource {
        match choice_value(&self.source, cx).as_deref() {
            Some("subscriptions") => ExpenseSource::Subscriptions,
            _ => ExpenseSource::Manual,
        }
    }

    fn subscriptions_expense(&self, period: PeriodType) -> i64 {
        self.subscriptions
            .iter()
            .map(|s| normalize_subscription_to_period(s.amount, &s.frequency, period))
            .sum()
    }

    /// The projection for the current form values, if they are valid.
    pub fn result(&self, cx: &gpui_kit::App) -> Option<(ProjectionResult, u32)> {
        let period_type = self.period(cx);
        let (starting, income, expense, periods) = (
            text_of(&self.starting, cx),
            text_of(&self.income, cx),
            text_of(&self.expense, cx),
            text_of(&self.periods, cx),
        );
        let input = build_input(&FormValues {
            starting_balance: &starting,
            income: &income,
            manual_expense: &expense,
            source: self.expense_source(cx),
            subscriptions_expense: self.subscriptions_expense(period_type),
            period_type,
            periods: &periods,
        })?;
        Some((compute_projection(input, chrono::Local::now().date_naive()), input.periods))
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
}

impl Render for ProjectionsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let period = self.period(cx);
        let source = self.expense_source(cx);
        let sub_total = self.subscriptions_expense(period);

        let expense_block = match source {
            ExpenseSource::Manual => field("Expense per Period", input("proj-expense", &self.expense, window, cx), cx).into_any_element(),
            ExpenseSource::Subscriptions => {
                let n = self.subscriptions.len();
                let mut list = div()
                    .flex()
                    .flex_col()
                    .gap(rems(0.25))
                    .p(rems(0.75))
                    .border_1()
                    .border_color(t.border)
                    .child(div().text_color(t.secondary).child(format!(
                        "{n} active subscription{}",
                        if n == 1 { "" } else { "s" }
                    )));
                for s in &self.subscriptions {
                    list = list.child(
                        div()
                            .flex()
                            .justify_between()
                            .child(s.name.clone())
                            .child(format!(
                                "{}/{}",
                                format_money(normalize_subscription_to_period(s.amount, &s.frequency, period)),
                                period.unit()
                            )),
                    );
                }
                list = list.child(div().font_weight(gpui_kit::FontWeight::BOLD).child(format!(
                    "Total: {}/{}",
                    format_money(sub_total),
                    period.unit()
                )));
                if n == 0 {
                    list = list.child(div().text_color(t.secondary).child("No active subscriptions found"));
                }
                list.into_any_element()
            }
        };

        let form = panel("Inputs", cx)
            .w(rems(20.))
            .flex_shrink_0()
            .child(field("Starting Balance", input("proj-start", &self.starting, window, cx), cx))
            .child(field("Income per Period", input("proj-income", &self.income, window, cx), cx))
            .child(field("Expense Source", select("proj-source", &self.source, window, cx), cx))
            .child(expense_block)
            .child(field("Period Type", select("proj-period", &self.period_type, window, cx), cx))
            .child(
                field(&format!("Number of {}", period.plural()), input("proj-periods", &self.periods, window, cx), cx)
                    .child(div().text_color(t.secondary).child(format!("1 – {} {}", period.limit(), period.plural()))),
            );

        let content = match self.result(cx) {
            None => empty_state(
                "Nothing to project yet",
                "Enter your income, expense and period to see a projection.",
                cx,
            )
            .into_any_element(),
            Some((result, periods)) => {
                let s = &result.summary;
                let color = |v: i64| if v > 0 { t.success } else if v < 0 { t.danger } else { t.foreground };
                let stat = |title: &str, value: String, note: String, tint| -> Div {
                    panel(title.to_string(), cx)
                        .flex_1()
                        .child(div().text_size(rems(1.25)).text_color(tint).child(value))
                        .child(div().text_color(t.secondary).child(note))
                };
                let stats = div()
                    .flex()
                    .gap(rems(1.))
                    .child(stat(
                        "Projected Balance",
                        format_balance(s.final_balance),
                        format!("in {periods} {} · {}", period.plural(), s.projected_date.format("%b %e, %Y")),
                        color(s.net_per_period),
                    ))
                    .child(stat(
                        "Total Change",
                        format_money(s.total_increase),
                        "from starting balance".into(),
                        color(s.total_increase),
                    ))
                    .child(stat(
                        &format!("Net per {}", capitalize(period.unit())),
                        format_money(s.net_per_period),
                        match s.retention_rate {
                            Some(r) => format!("Keeping {r:.1}% of income"),
                            None => "No income entered".into(),
                        },
                        color(s.net_per_period),
                    ));

                let declining = s.net_per_period < 0;
                let mut chart = LineChart::new(result.points.clone())
                    .id("projection-line")
                    .name("Balance")
                    .x(|p| SharedString::from(p.label.clone()))
                    .y(|p| p.balance as f64 / 100.0)
                    .stroke(if declining { t.danger } else { t.accent })
                    .tick_margin(2)
                    .linear()
                    .dot()
                    .y_tick_format(|v| format_axis((v * 100.0).round() as i64))
                    .tooltip_value(|_, v| format_balance((v * 100.0).round() as i64).into());
                if s.final_balance < 0 {
                    chart = chart.reference_line(0.0);
                }
                div()
                    .flex()
                    .flex_col()
                    .gap(rems(1.))
                    .child(stats)
                    .child(panel("Balance Over Time", cx).child(div().w_full().h(rems(20.)).child(chart)))
                    .into_any_element()
            }
        };

        div()
            .id("projections")
            .size_full()
            .overflow_y_scroll()
            .p(rems(1.5))
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(page_header("Projections", div().text_color(t.secondary).child("Project your balance growth over time"), cx))
            .child(div().flex().gap(rems(1.)).items_start().child(form).child(div().flex_1().min_w_0().child(content)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use chelete_lib::commands::CreateSubscriptionRequest;
    use gpui_kit::{TestAppContext, VisualTestContext};

    fn values<'a>(s: &'a str, i: &'a str, e: &'a str, n: &'a str) -> FormValues<'a> {
        FormValues {
            starting_balance: s,
            income: i,
            manual_expense: e,
            source: ExpenseSource::Manual,
            subscriptions_expense: 0,
            period_type: PeriodType::Months,
            periods: n,
        }
    }

    #[test]
    fn valid_form_builds_an_input() {
        let input = build_input(&values("0", "1200", "100", "10")).unwrap();
        assert_eq!((input.starting_balance, input.income, input.expense, input.periods), (0, 120000, 10000, 10));
    }

    #[test]
    fn any_empty_or_bad_field_blocks_the_projection() {
        assert!(build_input(&values("", "1200", "100", "10")).is_none());
        assert!(build_input(&values("0", "", "100", "10")).is_none());
        assert!(build_input(&values("0", "1200", "", "10")).is_none());
        assert!(build_input(&values("0", "1200", "100", "")).is_none());
        assert!(build_input(&values("0", "abc", "100", "10")).is_none());
    }

    #[test]
    fn period_count_is_bounded_by_the_period_type() {
        assert!(build_input(&values("0", "1", "1", "0")).is_none());
        assert!(build_input(&values("0", "1", "1", "601")).is_none());
        assert!(build_input(&values("0", "1", "1", "600")).is_some());
        assert!(build_input(&values("0", "1", "1", "2.5")).is_none());
    }

    #[test]
    fn subscription_source_ignores_the_manual_expense() {
        let mut v = values("0", "1000", "", "3");
        v.source = ExpenseSource::Subscriptions;
        v.subscriptions_expense = 4500;
        let input = build_input(&v).unwrap();
        assert_eq!(input.expense, 4500, "manual field may be blank in this mode");
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<ProjectionsScreen>, Arc<DbState>, &mut VisualTestContext) {
        init(cx);
        let db = seeded_db();
        let handle = db.clone();
        let (screen, vcx) = cx.add_window_view(move |window, cx| ProjectionsScreen::new(db, window, cx));
        (screen, handle, vcx)
    }

    fn type_into(input: &Entity<InputState>, text: &str, cx: &mut VisualTestContext) {
        cx.update(|window, cx| input.update(cx, |i, cx| i.set_value(text, window, cx)));
    }

    #[gpui_kit::test]
    fn typing_income_and_expense_produces_the_canonical_projection(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|_, cx| assert!(screen.read(cx).result(cx).is_none(), "income and expense start empty"));
        let (income, expense) = cx.update(|_, cx| (screen.read(cx).income.clone(), screen.read(cx).expense.clone()));
        type_into(&income, "1200", cx);
        type_into(&expense, "100", cx);
        cx.update(|_, cx| {
            let (result, periods) = screen.read(cx).result(cx).expect("valid inputs");
            assert_eq!(periods, 10);
            assert_eq!(result.summary.final_balance, 1_100_000);
            assert_eq!(result.points.len(), 11);
        });
    }

    #[gpui_kit::test]
    fn subscription_source_totals_active_subscriptions_only(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let make = |name: &str, amount: i64| {
            commands::create_subscription(&db, CreateSubscriptionRequest {
                name: name.into(), amount, currency: "USD".into(), frequency: "monthly".into(),
                category_id: None, account_id: None, start_date: None,
            }).unwrap()
        };
        make("Active", 1000);
        let inactive = make("Paused", 5000);
        commands::update_subscription(&db, commands::UpdateSubscriptionRequest {
            id: inactive.id, name: None, amount: None, currency: None, frequency: None,
            category_id: None, account_id: None, start_date: None, is_active: Some(false),
        }).unwrap();
        cx.update(|_, cx| screen.update(cx, |s, cx| s.reload(cx)));

        let (income, source) = cx.update(|_, cx| (screen.read(cx).income.clone(), screen.read(cx).source.clone()));
        type_into(&income, "100", cx);
        cx.update(|_, cx| source.update(cx, |c, cx| c.set_selected(Some(1), cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.subscriptions.len(), 1, "paused subscription excluded");
            let (result, _) = s.result(cx).expect("valid without a manual expense");
            assert_eq!(result.summary.net_per_period, 10000 - 1000);
        });
    }

    #[gpui_kit::test]
    fn switching_period_type_rescales_subscription_cost(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        commands::create_subscription(&db, CreateSubscriptionRequest {
            name: "Gym".into(), amount: 12000, currency: "USD".into(), frequency: "monthly".into(),
            category_id: None, account_id: None, start_date: None,
        }).unwrap();
        cx.update(|_, cx| screen.update(cx, |s, cx| s.reload(cx)));
        let period_type = cx.update(|_, cx| screen.read(cx).period_type.clone());
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.subscriptions_expense(PeriodType::Months), 12000);
            assert_eq!(s.subscriptions_expense(PeriodType::Weeks), 2769);
        });
        cx.update(|_, cx| period_type.update(cx, |c, cx| c.set_selected(Some(0), cx)));
        cx.update(|_, cx| assert_eq!(screen.read(cx).period(cx), PeriodType::Weeks));
    }
}
