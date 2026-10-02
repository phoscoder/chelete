use super::categories::parse_hex;
use super::date_filter_control::DateFilterControl;
use super::icons::category_icon;
use super::widgets::*;
use chelete_lib::commands::{self, Account, Category, Overview, Transaction};
use chelete_lib::database::DbState;
use chelete_lib::format::{format_balance, format_money};
use chelete_lib::overview::{filter_transactions, summarize, CategoryTotal, TypeFilter};
use chelete_lib::paging::{page_slice, Pager};
use gpui_kit::{
    div, prelude::*, px, rems, Context, Div, Entity, Hsla, IntoElement, Render, SharedString,
    Subscription, Window,
};
use gpui_omarchy::chart::PieChart;
use gpui_omarchy::{alert, empty_state, panel, select, ActiveTheme as _, ChoiceState, Status};
use std::sync::Arc;

const PER_PAGE: usize = 10;

pub struct OverviewScreen {
    db: Arc<DbState>,
    overview: Result<Overview, String>,
    transactions: Vec<Transaction>,
    accounts: Vec<Account>,
    categories: Vec<Category>,
    date_filter: DateFilterControl,
    kind: Entity<ChoiceState>,
    account: Entity<ChoiceState>,
    page: usize,
    _account_sub: Option<Subscription>,
    _subs: Vec<Subscription>,
}

impl OverviewScreen {
    pub fn new(db: Arc<DbState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let date_filter = DateFilterControl::new(window, cx);
        let kind = choice(
            vec![("all".into(), "All Types".into()), ("income".into(), "Income".into()), ("expense".into(), "Expense".into())],
            Some("all"),
            window,
            cx,
        );
        let account = choice(vec![("all".into(), "All Accounts".into())], Some("all"), window, cx);
        let mut subs = date_filter.observe(cx, |this: &mut Self, cx| {
            this.page = 1;
            cx.notify();
        });
        subs.push(cx.observe(&kind, |this, _, cx| {
            this.page = 1;
            cx.notify();
        }));
        let mut screen = Self {
            db,
            overview: Ok(empty_overview()),
            transactions: Vec::new(),
            accounts: Vec::new(),
            categories: Vec::new(),
            date_filter,
            kind,
            account,
            page: 1,
            _account_sub: None,
            _subs: subs,
        };
        screen.load(window, cx);
        screen
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.overview = commands::get_overview(&self.db);
        self.transactions = commands::get_transactions(&self.db).unwrap_or_default();
        self.accounts = commands::get_accounts(&self.db).unwrap_or_default();
        self.categories = commands::get_categories(&self.db).unwrap_or_default();

        // The account list can change between visits, so rebuild the dropdown
        // and keep the previous choice if that account still exists.
        let previous = choice_value(&self.account, cx).unwrap_or_else(|| "all".into());
        let mut items = vec![("all".to_string(), "All Accounts".to_string())];
        items.extend(self.accounts.iter().map(|a| (a.id.clone(), a.name.clone())));
        self.account = choice(items, Some(&previous), window, cx);
        self._account_sub = Some(cx.observe(&self.account, |this, _, cx| {
            this.page = 1;
            cx.notify();
        }));
    }

    pub fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.load(window, cx);
        cx.notify();
    }

    #[cfg(test)]
    pub fn overview(&self) -> Result<&Overview, &String> {
        self.overview.as_ref()
    }

    fn bounds(&self, cx: &gpui_kit::App) -> (String, String) {
        self.date_filter.bounds(chrono::Local::now().date_naive(), cx)
    }

    fn visible_transactions(&self, cx: &gpui_kit::App) -> Vec<&Transaction> {
        let (start, end) = self.bounds(cx);
        let kind = TypeFilter::from_value(&choice_value(&self.kind, cx).unwrap_or_default());
        let account = choice_value(&self.account, cx).filter(|a| a != "all");
        filter_transactions(&self.transactions, &start, &end, kind, account.as_deref())
    }
}

fn empty_overview() -> Overview {
    Overview {
        total_balance: 0,
        total_income: 0,
        total_expenses: 0,
        accounts: vec![],
        recent_transactions: vec![],
        category_spending: vec![],
    }
}

fn stat(title: &str, value: String, tint: Hsla, note: Option<String>, cx: &gpui_kit::App) -> Div {
    let card = panel(title.to_string(), cx)
        .flex_1()
        .child(div().text_size(rems(1.5)).text_color(tint).child(value));
    match note {
        Some(n) => card.child(dim(n, cx)),
        None => card,
    }
}

fn donut(title: &str, data: &[CategoryTotal], empty: &str, id: &'static str, cx: &gpui_kit::App) -> Div {
    let t = cx.omarchy();
    let slices: Vec<(SharedString, f32, Hsla)> = data
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let color = c.color.as_deref().and_then(parse_hex).unwrap_or(t.chart[i % t.chart.len()]);
            (SharedString::from(c.name.clone()), c.value as f32 / 100.0, color)
        })
        .collect();
    let body = if slices.is_empty() {
        div().py(rems(1.)).text_color(t.secondary).child(empty.to_string()).into_any_element()
    } else {
        div()
            .w_full()
            .h(rems(15.))
            .child(
                PieChart::new(slices)
                    .id(id)
                    .name("Total")
                    .value(|(_, v, _)| *v)
                    .color(|(_, _, c)| *c)
                    .inner_radius(48.)
                    .outer_radius(76.)
                    .pad_angle(0.02)
                    .label(|(name, v, _)| format!("{name} ${v:.0}").into()),
            )
            .into_any_element()
    };
    panel(title.to_string(), cx).flex_1().child(body)
}

impl Render for OverviewScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let page = div().id("overview").relative().size_full().overflow_y_scroll().p(rems(1.5)).flex().flex_col().gap(rems(1.));

        let overview = match &self.overview {
            Ok(o) => o,
            Err(e) => return page.child(alert(format!("Could not load overview: {e}"), Status::Error, cx)),
        };

        let (start, end) = self.bounds(cx);
        let summary = summarize(&self.transactions, &self.categories, &start, &end);
        let visible = self.visible_transactions(cx);
        let pager = Pager::new(self.page, PER_PAGE, visible.len());
        let rows: Vec<Transaction> = page_slice(&visible, &pager).iter().map(|t| (*t).clone()).collect();
        let nothing_at_all = self.accounts.is_empty() && self.transactions.is_empty();
        drop(visible);

        let today = chrono::Local::now().date_naive();
        let header = page_header("Overview", div().text_color(t.secondary).child(today.format("%B %Y").to_string()), cx);
        let filter_row = self.date_filter.render(today, window, cx);

        let net_tint = if summary.net >= 0 { t.success } else { t.danger };
        let stats = div()
            .flex()
            .gap(rems(1.))
            .child(stat("Total Balance", format_balance(overview.total_balance), t.foreground, None, cx))
            .child(stat("Income", format_money(summary.total_income), t.success, None, cx))
            .child(stat("Expenses", format_money(-summary.total_expenses), t.danger, None, cx))
            .child(stat("Net", format_money(summary.net), net_tint, Some(format!("Savings rate: {}%", summary.savings_rate)), cx));

        let accounts = (!self.accounts.is_empty()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(rems(0.5))
                .child(div().font_weight(gpui_kit::FontWeight::BOLD).child("Accounts"))
                .child(div().flex().flex_wrap().gap(rems(1.)).children(self.accounts.iter().map(|a| {
                    panel(a.name.clone(), cx).w(rems(14.)).child(div().text_size(rems(1.25)).child(format_balance(a.balance)))
                })))
        });

        let breakdown = div()
            .flex()
            .flex_col()
            .gap(rems(0.5))
            .child(div().font_weight(gpui_kit::FontWeight::BOLD).child("Breakdown"))
            .child(
                div()
                    .flex()
                    .gap(rems(1.))
                    .child(donut("Income", &summary.income_by_category, "No income for this period", "overview-income", cx))
                    .child(donut("Expenses", &summary.expenses_by_category, "No expenses for this period", "overview-expenses", cx)),
            );

        let list = if rows.is_empty() {
            div().py(rems(1.)).text_color(t.secondary).child("No transactions match the selected filters.").into_any_element()
        } else {
            let mut table = div().flex().flex_col().border_1().border_color(t.border).child(
                div()
                    .flex()
                    .py(rems(0.375))
                    .bg(t.surface)
                    .text_color(t.bright)
                    .font_weight(gpui_kit::FontWeight::BOLD)
                    .child(cell(Some(6.), "Date"))
                    .child(cell(None, "Description"))
                    .child(cell(None, "Category"))
                    .child(cell(Some(9.), div().flex().justify_end().child("Amount"))),
            );
            for tx in rows {
                let category = tx.category_id.as_ref().and_then(|id| self.categories.iter().find(|c| &c.id == id));
                let tint = if tx.transaction_type == "income" { t.success } else { t.danger };
                let amount = if tx.transaction_type == "expense" { -tx.amount } else { tx.amount };
                let date = chrono::NaiveDate::parse_from_str(&tx.transaction_date, "%Y-%m-%d")
                    .map(|d| d.format("%b %e").to_string())
                    .unwrap_or_else(|_| tx.transaction_date.clone());
                table = table.child(
                    div()
                        .flex()
                        .items_center()
                        .py(rems(0.25))
                        .border_t_1()
                        .border_color(t.border)
                        .child(cell(Some(6.), date))
                        .child(cell(None, tx.description.clone()))
                        .child(cell(None, match category {
                            Some(c) => div()
                                .flex()
                                .items_center()
                                .gap(rems(0.25))
                                .children(c.icon.as_deref().map(|n| category_icon(n, px(12.), t.secondary)))
                                .child(c.name.clone())
                                .into_any_element(),
                            None => dim("—", cx).into_any_element(),
                        }))
                        .child(cell(Some(9.), div().flex().justify_end().text_color(tint).child(format_money(amount)))),
                );
            }
            let screen = cx.entity();
            let (prev, next) = (screen.clone(), screen);
            div()
                .flex()
                .flex_col()
                .gap(rems(0.5))
                .child(table)
                .child(simple_pager(
                    pager,
                    move |_, _, cx| prev.update(cx, |s, cx| { s.page = s.page.saturating_sub(1).max(1); cx.notify(); }),
                    move |_, _, cx| next.update(cx, |s, cx| { s.page += 1; cx.notify(); }),
                    cx,
                ))
                .into_any_element()
        };

        let recent = div()
            .flex()
            .flex_col()
            .gap(rems(0.5))
            .child(div().font_weight(gpui_kit::FontWeight::BOLD).child("Recent Transactions"))
            .child(
                div()
                    .flex()
                    .gap(rems(0.5))
                    .child(div().w(rems(13.)).child(select("overview-kind", &self.kind, window, cx)))
                    .child(div().w(rems(13.)).child(select("overview-account", &self.account, window, cx))),
            )
            .child(list);

        let welcome = nothing_at_all.then(|| {
            empty_state("Welcome to Chelete", "Press Ctrl+K to open the command palette and add your first account.", cx)
        });

        page.child(header)
            .child(filter_row)
            .child(stats)
            .children(accounts)
            .child(breakdown)
            .child(recent)
            .children(welcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{empty_db, init, seeded_db};
    use chelete_lib::date_filter::DateFilter;
    use gpui_kit::{TestAppContext, VisualTestContext};

    fn setup(cx: &mut TestAppContext, db: Arc<DbState>) -> (Entity<OverviewScreen>, &mut VisualTestContext) {
        init(cx);
        cx.add_window_view(move |window, cx| OverviewScreen::new(db, window, cx))
    }

    fn pick(choice: &Entity<ChoiceState>, index: usize, cx: &mut VisualTestContext) {
        cx.update(|_, cx| choice.update(cx, |c, cx| c.set_selected(Some(index), cx)));
    }

    #[gpui_kit::test]
    fn loads_totals_from_seeded_data(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx, seeded_db());
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let o = s.overview().expect("loads");
            assert_eq!(o.accounts.len(), 4);
            assert_eq!(s.visible_transactions(cx).len(), 25, "All Time shows everything");
            let (start, end) = s.bounds(cx);
            let summary = summarize(&s.transactions, &s.categories, &start, &end);
            let sum = |kind: &str| s.transactions.iter().filter(|t| t.transaction_type == kind).map(|t| t.amount).sum::<i64>();
            // The summary follows the chosen range (all time here); the backend's own
            // income/expense figures are current-month only and are not shown.
            assert_eq!(summary.total_income, sum("income"));
            assert_eq!(summary.total_expenses, sum("expense"));
            let balance: i64 = s.accounts.iter().map(|a| a.balance).sum();
            assert_eq!(o.total_balance, balance, "total balance comes from the accounts");
        });
    }

    #[gpui_kit::test]
    fn type_and_account_filters_narrow_the_table(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx, seeded_db());
        let (kind, account) = cx.update(|_, cx| (screen.read(cx).kind.clone(), screen.read(cx).account.clone()));
        pick(&kind, 1, cx); // Income
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let shown = s.visible_transactions(cx);
            assert!(!shown.is_empty() && shown.iter().all(|t| t.transaction_type == "income"));
        });
        pick(&kind, 0, cx);
        pick(&account, 1, cx); // first real account
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let wanted = s.accounts[0].id.clone();
            let shown = s.visible_transactions(cx);
            assert!(!shown.is_empty() && shown.iter().all(|t| t.account_id == wanted));
        });
    }

    #[gpui_kit::test]
    fn date_filter_changes_the_summary_window(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx, seeded_db());
        let preset = cx.update(|_, cx| screen.read(cx).date_filter.preset.clone());
        let index = DateFilter::ALL_FILTERS.iter().position(|f| *f == DateFilter::Today).unwrap();
        cx.update(|_, cx| screen.update(cx, |s, cx| { s.page = 2; cx.notify(); }));
        pick(&preset, index, cx);
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.page, 1);
            let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
            assert!(s.visible_transactions(cx).iter().all(|t| t.transaction_date == today));
        });
    }

    #[gpui_kit::test]
    fn pages_hold_ten_rows(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx, seeded_db());
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let pager = Pager::new(1, PER_PAGE, s.visible_transactions(cx).len());
            assert_eq!(pager.total_pages(), 3);
            assert_eq!(pager.label(), "1–10 of 25");
        });
    }

    #[gpui_kit::test]
    fn reload_picks_up_new_accounts_and_keeps_the_selection(cx: &mut TestAppContext) {
        let db = seeded_db();
        let (screen, cx) = setup(cx, db.clone());
        let account = cx.update(|_, cx| screen.read(cx).account.clone());
        pick(&account, 2, cx);
        let chosen = cx.update(|_, cx| choice_value(&screen.read(cx).account, cx));
        commands::create_account(
            &db,
            commands::CreateAccountRequest { name: "New".into(), account_type: "cash".into(), currency: "USD".into(), balance: 0, color: None, icon: None },
        )
        .unwrap();
        cx.update(|window, cx| screen.update(cx, |s, cx| s.reload(window, cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.accounts.len(), 5);
            assert_eq!(choice_value(&s.account, cx), chosen, "selection survives the rebuild");
        });
    }

    #[gpui_kit::test]
    fn an_empty_database_shows_the_welcome_state_without_errors(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx, empty_db());
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(s.overview().is_ok());
            assert!(s.accounts.is_empty() && s.transactions.is_empty());
            assert!(s.visible_transactions(cx).is_empty());
        });
    }
}
