//! Search box plus type, account and category dropdowns for the transactions
//! list. Rebuilt whenever the accounts or categories change, keeping the
//! current selection where it still exists.
use super::date_filter_control::PRESET_WIDTH;
use super::widgets::{choice, choice_value, text_input_with, text_of};
use chelete_lib::commands::{Account, Category};
use chelete_lib::transaction_filter::{TransactionFilter, UNCATEGORIZED};
use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::{div, prelude::*, rems, App, Context, Div, Entity, Subscription, Window};
use gpui_omarchy::{icon, input, select, ActiveTheme as _, ChoiceState, IconName};

const ALL: &str = "all";

pub struct TransactionFilterControl {
    pub search: Entity<InputState>,
    pub kind: Entity<ChoiceState>,
    pub account: Entity<ChoiceState>,
    pub category: Entity<ChoiceState>,
    /// The account and category lists the dropdowns were built from.
    options: Vec<(String, String)>,
}

impl TransactionFilterControl {
    /// Fresh controls with `keep` pre-selected where it still applies.
    pub fn new<V: 'static>(
        keep: &TransactionFilter,
        accounts: &[Account],
        categories: &[Category],
        window: &mut Window,
        cx: &mut Context<V>,
    ) -> Self {
        let kinds = vec![
            (ALL.to_string(), "All Types".to_string()),
            ("income".to_string(), "Income".to_string()),
            ("expense".to_string(), "Expense".to_string()),
        ];
        let mut account_items = vec![(ALL.to_string(), "All Accounts".to_string())];
        account_items.extend(accounts.iter().map(|a| (a.id.clone(), a.name.clone())));
        let mut category_items = vec![
            (ALL.to_string(), "All Categories".to_string()),
            (UNCATEGORIZED.to_string(), "Uncategorized".to_string()),
        ];
        category_items.extend(categories.iter().map(|c| (c.id.clone(), c.name.clone())));

        let pick = |items: &[(String, String)], value: &Option<String>| -> String {
            value.clone().filter(|v| items.iter().any(|(id, _)| id == v)).unwrap_or_else(|| ALL.to_string())
        };
        let kind = pick(&kinds, &keep.kind);
        let account = pick(&account_items, &keep.account_id);
        let category = pick(&category_items, &keep.category_id);
        Self {
            search: text_input_with("Search transactions", &keep.search, window, cx),
            kind: choice(kinds, Some(&kind), window, cx),
            options: options(accounts, categories),
            account: choice(account_items, Some(&account), window, cx),
            category: choice(category_items, Some(&category), window, cx),
        }
    }

    /// Whether the dropdowns are out of date with these lists.
    pub fn is_stale(&self, accounts: &[Account], categories: &[Category]) -> bool {
        self.options != options(accounts, categories)
    }

    pub fn filter(&self, cx: &App) -> TransactionFilter {
        let picked = |state: &Entity<ChoiceState>| choice_value(state, cx).filter(|v| v != ALL);
        TransactionFilter {
            search: text_of(&self.search, cx),
            kind: picked(&self.kind),
            account_id: picked(&self.account),
            category_id: picked(&self.category),
        }
    }

    /// Re-render `owner` whenever the search text or a dropdown changes.
    pub fn observe<V: 'static>(&self, cx: &mut Context<V>, on_change: impl Fn(&mut V, &mut Context<V>) + Clone + 'static) -> Vec<Subscription> {
        let typed = on_change.clone();
        let mut subs = vec![cx.subscribe(&self.search, move |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                typed(this, cx);
            }
        })];
        for state in [&self.kind, &self.account, &self.category] {
            let changed = on_change.clone();
            subs.push(cx.observe(state, move |this, _, cx| changed(this, cx)));
        }
        subs
    }

    pub fn render(&self, window: &mut Window, cx: &mut App) -> Div {
        div()
            .flex()
            .items_center()
            .gap(rems(0.5))
            .child(
                // Same size as the date dropdown, stretched to its height so they line up.
                div().w(rems(PRESET_WIDTH)).self_stretch().child(
                    input("tx-search", &self.search, window, cx)
                        .debug_selector(|| "tx-search".into())
                        .h_full()
                        .py(rems(0.))
                        .prefix(icon(IconName::Search).size(rems(0.875)).text_color(cx.omarchy().secondary)),
                ),
            )
            .child(div().w(rems(9.)).child(select("tx-kind", &self.kind, window, cx)))
            .child(div().w(rems(11.)).child(select("tx-account", &self.account, window, cx)))
            .child(div().w(rems(11.)).child(select("tx-category", &self.category, window, cx)))
    }
}

fn options(accounts: &[Account], categories: &[Category]) -> Vec<(String, String)> {
    accounts
        .iter()
        .map(|a| (a.id.clone(), a.name.clone()))
        .chain(categories.iter().map(|c| (c.id.clone(), c.name.clone())))
        .collect()
}
