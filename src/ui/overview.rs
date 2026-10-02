use chelete_lib::commands::{self, Overview};
use chelete_lib::database::DbState;
use chelete_lib::format::{account_type_label, format_balance, format_money};
use gpui_kit::{div, prelude::*, rems, App, Div, IntoElement, SharedString};
use gpui_omarchy::chart::PieChart;
use gpui_omarchy::{alert, empty_state, panel, ActiveTheme as _, Status};

pub struct OverviewData {
    pub result: Result<Overview, String>,
}

impl OverviewData {
    pub fn load(db: &DbState) -> Self {
        Self {
            result: commands::get_overview(db),
        }
    }
}

fn stat(title: &'static str, value: String, color: gpui_kit::Hsla, cx: &App) -> Div {
    panel(title, cx)
        .flex_1()
        .child(div().text_size(rems(1.5)).text_color(color).child(value))
}

pub fn render(data: &OverviewData, cx: &App) -> impl IntoElement {
    let t = cx.omarchy();
    let page = div()
        .id("overview")
        .size_full()
        .overflow_y_scroll()
        .p(rems(1.5))
        .flex()
        .flex_col()
        .gap(rems(1.));

    let overview = match &data.result {
        Ok(o) => o,
        Err(e) => {
            return page.child(alert(format!("Could not load overview: {e}"), Status::Error, cx))
        }
    };

    let stats = div()
        .flex()
        .gap(rems(1.))
        .child(stat("Balance", format_balance(overview.total_balance), t.foreground, cx))
        .child(stat("Income", format_money(overview.total_income), t.success, cx))
        .child(stat("Expenses", format_money(-overview.total_expenses), t.danger, cx));

    let spending: Vec<(SharedString, f32)> = overview
        .category_spending
        .iter()
        .filter(|c| c.spent > 0)
        .map(|c| (SharedString::from(c.category_name.clone()), c.spent as f32 / 100.))
        .collect();

    let spending_panel = panel("Spending by category", cx).flex_1().child(if spending.is_empty() {
        empty_state("No spending yet", "Expenses will appear here.", cx).into_any_element()
    } else {
        div()
            .w_full()
            .h(rems(16.))
            .child(
                PieChart::new(spending)
                    .id("overview-spending")
                    .name("Spent")
                    .value(|(_, v)| *v)
                    .inner_radius(56.)
                    .outer_radius(88.)
                    .pad_angle(0.02)
                    .label(|(name, v)| format!("{name} ${v:.0}").into()),
            )
            .into_any_element()
    });

    let accounts = panel("Accounts", cx).flex_1().children(overview.accounts.iter().map(|a| {
        div()
            .flex()
            .justify_between()
            .child(
                div()
                    .child(a.name.clone())
                    .child(div().text_color(t.secondary).child(account_type_label(&a.account_type))),
            )
            .child(format_balance(a.balance))
    }));

    let recent = panel("Recent transactions", cx).children(
        overview.recent_transactions.iter().map(|tx| {
            let color = if tx.transaction_type == "income" { t.success } else { t.danger };
            let signed = if tx.transaction_type == "income" { tx.amount } else { -tx.amount };
            div()
                .flex()
                .justify_between()
                .child(div().flex().gap(rems(1.)).child(div().text_color(t.secondary).child(tx.transaction_date.clone())).child(tx.description.clone()))
                .child(div().text_color(color).child(format_money(signed)))
        }),
    );

    page.child(stats)
        .child(div().flex().gap(rems(1.)).child(spending_panel).child(accounts))
        .child(recent)
}
