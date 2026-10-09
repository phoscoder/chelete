use super::accounts::FormEvent;
use super::date_field;
use super::icons::category_icon;
use super::widgets::*;
use chelete_lib::commands::{
    self, Account, Category, CreateSubscriptionRequest, CreateTransactionRequest, Subscription,
    UpdateSubscriptionRequest,
};
use chelete_lib::database::DbState;
use chelete_lib::format::{format_money, parse_cents};
use chelete_lib::paging::{page_slice, Pager};
use chelete_lib::subscriptions::{filter_by_frequency, frequency_label, next_payment_date, total_amount, FREQUENCIES};
use gpui_kit::base::input::InputState;
use gpui_kit::component::date_picker::{DatePicker, DatePickerState};
use gpui_kit::{
    div, prelude::*, px, rems, App, Context, Entity, EventEmitter, IntoElement, Render,
    SharedString, Subscription as GpuiSubscription, Window,
};
use gpui_omarchy::{
    button, icon_button, input, panel, select, ActiveTheme as _, ButtonVariant, ChoiceState, IconName,
};
use std::collections::HashSet;
use std::sync::Arc;

enum Dialog {
    Form(Entity<SubscriptionForm>),
    View(Subscription),
    Confirm(Entity<ConfirmDialog>, Vec<String>),
}

pub struct SubscriptionsScreen {
    db: Arc<DbState>,
    subscriptions: Vec<Subscription>,
    categories: Vec<Category>,
    accounts: Vec<Account>,
    filter: Entity<ChoiceState>,
    applied_filter: String,
    per_page: Entity<ChoiceState>,
    applied_per_page: usize,
    page: usize,
    selected: HashSet<String>,
    dialog: Option<Dialog>,
    view_focus: gpui_kit::FocusHandle,
    _dialog_sub: Option<GpuiSubscription>,
    _subs: Vec<GpuiSubscription>,
}

impl EventEmitter<ScreenEvent> for SubscriptionsScreen {}

impl SubscriptionsScreen {
    pub fn new(db: Arc<DbState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut items = vec![("all".to_string(), "All Periods".to_string())];
        items.extend(FREQUENCIES.iter().map(|(v, l)| (v.to_string(), l.to_string())));
        let filter = choice(items, Some("all"), window, cx);
        let per_page = per_page_choice(25, window, cx);
        let subs = vec![
            cx.observe(&filter, |this, _, cx| this.on_controls_changed(cx)),
            cx.observe(&per_page, |this, _, cx| this.on_controls_changed(cx)),
        ];
        let mut screen = Self {
            db,
            subscriptions: Vec::new(),
            categories: Vec::new(),
            accounts: Vec::new(),
            filter,
            applied_filter: "all".into(),
            per_page,
            applied_per_page: 25,
            page: 1,
            selected: HashSet::new(),
            dialog: None,
            view_focus: cx.focus_handle(),
            _dialog_sub: None,
            _subs: subs,
        };
        screen.load();
        screen
    }

    fn load(&mut self) {
        self.subscriptions = commands::get_subscriptions(&self.db).unwrap_or_default();
        self.categories = commands::get_categories(&self.db).unwrap_or_default();
        self.accounts = commands::get_accounts(&self.db).unwrap_or_default();
        let live: HashSet<&String> = self.subscriptions.iter().map(|s| &s.id).collect();
        self.selected.retain(|id| live.contains(id));
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load();
        cx.notify();
    }

    fn on_controls_changed(&mut self, cx: &mut Context<Self>) {
        let filter = choice_value(&self.filter, cx).unwrap_or_else(|| "all".into());
        let per_page = choice_value(&self.per_page, cx)
            .and_then(|v| v.parse().ok())
            .unwrap_or(self.applied_per_page);
        if filter != self.applied_filter || per_page != self.applied_per_page {
            self.applied_filter = filter;
            self.applied_per_page = per_page;
            self.page = 1;
        }
        cx.notify();
    }

    #[cfg(test)]
    pub fn subscriptions(&self) -> &[Subscription] {
        &self.subscriptions
    }

    fn filtered(&self) -> Vec<&Subscription> {
        filter_by_frequency(&self.subscriptions, &self.applied_filter)
    }

    fn pager(&self) -> Pager {
        Pager::new(self.page, self.applied_per_page, self.filtered().len())
    }

    #[cfg(test)]
    fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn open_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_form(None, window, cx);
    }

    fn open_form(&mut self, editing: Option<Subscription>, window: &mut Window, cx: &mut Context<Self>) {
        let form = cx.new(|cx| SubscriptionForm::new(self.db.clone(), &self.categories, &self.accounts, editing, window, cx));
        self._dialog_sub = Some(cx.subscribe(&form, |this, _, event: &FormEvent, cx| {
            match event {
                FormEvent::Cancel => {}
                FormEvent::Saved(message) => {
                    this.load();
                    cx.emit(ScreenEvent::ok(message.clone()));
                }
                FormEvent::Failed(message) => {
                    cx.emit(ScreenEvent::error(message.clone()));
                    return;
                }
            }
            this.close_dialog(cx);
        }));
        self.dialog = Some(Dialog::Form(form));
        cx.notify();
    }

    fn close_dialog(&mut self, cx: &mut Context<Self>) {
        self.dialog = None;
        self._dialog_sub = None;
        cx.notify();
    }

    fn ask_delete(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        if ids.is_empty() {
            return;
        }
        let noun = plural(ids.len(), "subscription");
        let dialog = cx.new(|cx| {
            ConfirmDialog::new(
                format!("Delete {noun}?"),
                format!(
                    "This will permanently remove the selected subscription{}. This action cannot be undone.",
                    if ids.len() == 1 { "" } else { "s" }
                ),
                "Delete",
                cx,
            )
        });
        self._dialog_sub = Some(cx.subscribe(&dialog, |this, _, event: &ConfirmEvent, cx| {
            if *event == ConfirmEvent::Confirm {
                if let Some(Dialog::Confirm(_, ids)) = this.dialog.take() {
                    this.delete(ids, cx);
                }
            }
            this.close_dialog(cx);
        }));
        self.dialog = Some(Dialog::Confirm(dialog, ids));
        cx.notify();
    }

    fn delete(&mut self, ids: Vec<String>, cx: &mut Context<Self>) {
        match commands::delete_subscriptions(&self.db, ids.clone()) {
            Ok(()) => {
                for id in &ids {
                    self.selected.remove(id);
                }
                self.load();
                let pager = self.pager();
                self.page = pager.page;
                cx.emit(ScreenEvent::ok(format!("Deleted {}", plural(ids.len(), "subscription"))));
            }
            Err(e) => cx.emit(ScreenEvent::error(format!("Could not delete: {e}"))),
        }
    }

    fn pay(&mut self, sub: &Subscription, cx: &mut Context<Self>) {
        let Some(account_id) = sub.account_id.clone().or_else(|| self.accounts.first().map(|a| a.id.clone())) else {
            return cx.emit(ScreenEvent::error("Add an account before recording a payment"));
        };
        let result = commands::create_transaction(
            &self.db,
            CreateTransactionRequest {
                account_id,
                category_id: sub.category_id.clone(),
                transaction_type: "expense".into(),
                amount: sub.amount,
                currency: sub.currency.clone(),
                description: sub.name.clone(),
                merchant: None,
                notes: None,
                transaction_date: chrono::Local::now().date_naive().format("%Y-%m-%d").to_string(),
            },
        );
        match result {
            Ok(_) => {
                self.load();
                cx.emit(ScreenEvent::ok(format!("Payment recorded for {}", sub.name)));
            }
            Err(_) => cx.emit(ScreenEvent::error("Failed to record payment")),
        }
        cx.notify();
    }

    fn set_active(&mut self, sub: &Subscription, active: bool, cx: &mut Context<Self>) {
        let result = commands::update_subscription(
            &self.db,
            UpdateSubscriptionRequest {
                id: sub.id.clone(),
                name: None,
                amount: None,
                currency: None,
                frequency: None,
                category_id: None,
                account_id: None,
                start_date: None,
                is_active: Some(active),
            },
        );
        match result {
            Ok(_) => {
                self.load();
                let state = if active { "active" } else { "inactive" };
                cx.emit(ScreenEvent::ok(format!("Marked {} as {state}", sub.name)));
            }
            Err(e) => cx.emit(ScreenEvent::error(format!("Could not update subscription: {e}"))),
        }
        cx.notify();
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.selected.remove(id) {
            self.selected.insert(id.to_string());
        }
        cx.notify();
    }

    fn toggle_page(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered();
        let ids: Vec<String> = page_slice(&filtered, &self.pager()).iter().map(|s| s.id.clone()).collect();
        let all_selected = !ids.is_empty() && ids.iter().all(|id| self.selected.contains(id));
        for id in ids {
            if all_selected {
                self.selected.remove(&id);
            } else {
                self.selected.insert(id);
            }
        }
        cx.notify();
    }

    fn category_name(&self, id: &Option<String>) -> Option<&Category> {
        id.as_ref().and_then(|id| self.categories.iter().find(|c| &c.id == id))
    }

    fn account_name(&self, id: &Option<String>) -> Option<&str> {
        id.as_ref()
            .and_then(|id| self.accounts.iter().find(|a| &a.id == id))
            .map(|a| a.name.as_str())
    }
}

impl Render for SubscriptionsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let today = chrono::Local::now().date_naive();
        let filtered = self.filtered();
        let pager = self.pager();
        let rows: Vec<Subscription> = page_slice(&filtered, &pager).iter().map(|s| (*s).clone()).collect();
        let total = total_amount(&filtered);
        let count = filtered.len();
        let selected_on_page = rows.iter().filter(|s| self.selected.contains(&s.id)).count();
        let header_state = selection_state(selected_on_page, rows.len());
        let filter_label = if self.applied_filter == "all" { "All Periods".to_string() } else { frequency_label(&self.applied_filter) };

        let actions = if self.selected.is_empty() {
            button("add-subscription", "+ Add", ButtonVariant::Primary, cx)
                .on_click(cx.listener(|this, _, window, cx| this.open_add(window, cx)))
                .into_any_element()
        } else {
            let ids: Vec<String> = self.selected.iter().cloned().collect();
            button("delete-selected", format!("Delete {} selected", self.selected.len()), ButtonVariant::Danger, cx)
                .on_click(cx.listener(move |this, _, _, cx| this.ask_delete(ids.clone(), cx)))
                .into_any_element()
        };

        let stats = div()
            .flex()
            .gap(rems(1.))
            .child(panel(format!("Total ({filter_label})"), cx).flex_1().child(
                div().text_size(rems(1.5)).child(format_money(total)),
            ))
            .child(panel("Subscriptions", cx).flex_1().child(div().text_size(rems(1.5)).child(count.to_string())));

        let table = if rows.is_empty() {
            div().py(rems(3.)).flex().justify_center().text_color(t.secondary).child("No subscriptions found.").into_any_element()
        } else {
            let header = div()
                .flex()
                .items_center()
                .py(rems(0.375))
                .bg(t.surface)
                .text_color(t.bright)
                .font_weight(gpui_kit::FontWeight::BOLD)
                .child(cell(Some(2.5), check("select-page", "", header_state, {
                    let screen = cx.entity();
                    move |_, cx| screen.update(cx, |s, cx| s.toggle_page(cx))
                }, cx)))
                .child(cell(Some(1.5), ""))
                .child(cell(None, "Name"))
                .child(cell(None, "Category"))
                .child(cell(None, "Account"))
                .child(cell(Some(7.), "Period"))
                .child(cell(Some(7.), "Start Date"))
                .child(cell(Some(7.), "Next Payment"))
                .child(cell(Some(7.), div().flex().justify_end().child("Amount")))
                .child(cell(Some(9.5), ""));

            let mut list = div().flex().flex_col().border_1().border_color(t.border).child(header);
            for s in rows {
                let next = next_payment_date(s.start_date.as_deref(), &s.frequency, today)
                    .map(|d| d.format("%Y-%m-%d").to_string());
                let category = self.category_name(&s.category_id).cloned();
                let account = self.account_name(&s.account_id).map(str::to_string);
                let checked = self.selected.contains(&s.id);
                let row_id = s.id.clone();
                let key = |prefix: &str| SharedString::from(format!("{prefix}-{}", s.id));
                let (view, edit, pay, del, status) = (s.clone(), s.clone(), s.clone(), s.id.clone(), s.clone());
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .py(rems(0.25))
                        .border_t_1()
                        .border_color(t.border)
                        .hover(|st| st.bg(t.surface))
                        .child(cell(Some(2.5), check(key("sel"), "", if checked { gpui_kit::base::CheckboxState::Checked } else { gpui_kit::base::CheckboxState::Unchecked }, {
                            let screen = cx.entity();
                            move |_, cx| screen.update(cx, |s, cx| s.toggle(&row_id, cx))
                        }, cx)))
                        .child(cell(Some(1.5), status_dot(s.is_active, cx)
                            .id(key("status"))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| this.set_active(&status, !status.is_active, cx)))))
                        .child(cell(None, s.name.clone()))
                        .child(cell(None, match &category {
                            Some(c) => div()
                                .flex()
                                .items_center()
                                .gap(rems(0.25))
                                .children(c.icon.as_deref().map(|n| category_icon(n, px(12.), t.secondary)))
                                .child(c.name.clone())
                                .into_any_element(),
                            None => dim("—", cx).into_any_element(),
                        }))
                        .child(cell(None, account.map(|a| div().child(a)).unwrap_or_else(|| dim("—", cx))))
                        .child(cell(Some(7.), frequency_label(&s.frequency)))
                        .child(cell(Some(7.), s.start_date.clone().map(|d| div().child(d)).unwrap_or_else(|| dim("—", cx))))
                        .child(cell(Some(7.), next.map(|d| div().child(d)).unwrap_or_else(|| dim("—", cx))))
                        .child(cell(Some(7.), div().flex().justify_end().child(format_money(s.amount))))
                        .child(cell(
                            Some(9.5),
                            div()
                                .flex()
                                .justify_end()
                                .gap(rems(0.125))
                                .child(icon_button(key("view"), IconName::Eye, "View subscription", ButtonVariant::Secondary, cx)
                                    .on_click(cx.listener(move |this, _, window, cx| { this.dialog = Some(Dialog::View(view.clone())); this.view_focus.focus(window, cx); cx.notify(); })))
                                .child(icon_button(key("edit"), IconName::Pencil, "Edit subscription", ButtonVariant::Secondary, cx)
                                    .on_click(cx.listener(move |this, _, window, cx| this.open_form(Some(edit.clone()), window, cx))))
                                .child(icon_button(key("pay"), IconName::CreditCard, "Record payment", ButtonVariant::Secondary, cx)
                                    .on_click(cx.listener(move |this, _, _, cx| this.pay(&pay.clone(), cx))))
                                .child(icon_button(key("del"), IconName::Trash, "Delete subscription", ButtonVariant::Danger, cx)
                                    .on_click(cx.listener(move |this, _, _, cx| this.ask_delete(vec![del.clone()], cx)))),
                        )),
                );
            }
            let screen = cx.entity();
            let (prev, next) = (screen.clone(), screen);
            div()
                .flex()
                .flex_col()
                .gap(rems(0.75))
                .child(list)
                .child(pagination_bar(
                    pager,
                    &self.per_page,
                    move |_, _, cx| prev.update(cx, |s, cx| { s.page = s.page.saturating_sub(1).max(1); cx.notify(); }),
                    move |_, _, cx| next.update(cx, |s, cx| { s.page += 1; cx.notify(); }),
                    window,
                    cx,
                ))
                .into_any_element()
        };

        let dialog = self.dialog.as_ref().map(|d| match d {
            Dialog::Form(f) => f.clone().into_any_element(),
            Dialog::Confirm(c, _) => c.clone().into_any_element(),
            Dialog::View(s) => self.view_dialog(s, cx).into_any_element(),
        });

        div()
            .id("subscriptions")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .p(rems(1.5))
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(page_header("Subscriptions", actions, cx))
            .child(div().w(rems(14.)).child(select("sub-filter", &self.filter, window, cx)))
            .child(stats)
            .child(table)
            .children(dialog)
    }
}

impl SubscriptionsScreen {
    fn view_dialog(&self, s: &Subscription, cx: &mut Context<Self>) -> impl IntoElement {
        let today = chrono::Local::now().date_naive();
        let next = next_payment_date(s.start_date.as_deref(), &s.frequency, today)
            .map(|d| d.format("%Y-%m-%d").to_string());
        let category = self.category_name(&s.category_id).map(|c| c.name.clone());
        let account = self.account_name(&s.account_id).map(str::to_string);
        let row = |label: &str, value: String| {
            div()
                .flex()
                .justify_between()
                .child(dim(label.to_string(), cx))
                .child(value)
        };
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.5))
            .child(row("Name", s.name.clone()))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(dim("Status", cx))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(rems(0.375))
                            .child(status_dot(s.is_active, cx))
                            .child(if s.is_active { "Active" } else { "Inactive" }),
                    ),
            )
            .child(row("Amount", format_money(s.amount)))
            .child(row("Period", frequency_label(&s.frequency)))
            .child(row("Start Date", s.start_date.clone().unwrap_or_else(|| "—".into())))
            .child(row("Next Payment", next.unwrap_or_else(|| "—".into())))
            .child(row("Account", account.unwrap_or_else(|| "—".into())))
            .child(row("Category", category.unwrap_or_else(|| "—".into())))
            .child(
                div().flex().justify_end().pt(rems(0.5)).child(
                    button("view-close", "Close", ButtonVariant::Primary, cx)
                        .on_click(cx.listener(|this, _, _, cx| this.close_dialog(cx))),
                ),
            );
        modal("Subscription", body, cx.listener(|this, _, _, cx| this.close_dialog(cx)), cx).track_focus(&self.view_focus)
            .on_action(cx.listener(|this, _: &super::ModalCancel, _, cx| this.close_dialog(cx)))
    }
}

pub struct SubscriptionForm {
    db: Arc<DbState>,
    editing: Option<String>,
    name: Entity<InputState>,
    amount: Entity<InputState>,
    frequency: Entity<ChoiceState>,
    category: Entity<ChoiceState>,
    account: Entity<ChoiceState>,
    start_date: Entity<DatePickerState>,
    _subs: Vec<GpuiSubscription>,
}

impl EventEmitter<FormEvent> for SubscriptionForm {}

impl SubscriptionForm {
    fn new(
        db: Arc<DbState>,
        categories: &[Category],
        accounts: &[Account],
        editing: Option<Subscription>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut category_items = vec![(String::new(), "None".to_string())];
        category_items.extend(categories.iter().filter(|c| c.category_type == "expense").map(|c| (c.id.clone(), c.name.clone())));
        let mut account_items = vec![(String::new(), "None".to_string())];
        account_items.extend(accounts.iter().map(|a| (a.id.clone(), a.name.clone())));
        let frequency_items = FREQUENCIES.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect();

        let (name, amount, frequency, category, account, start) = match &editing {
            Some(s) => (
                s.name.clone(),
                format!("{:.2}", s.amount as f64 / 100.0),
                s.frequency.clone(),
                s.category_id.clone().unwrap_or_default(),
                s.account_id.clone().unwrap_or_default(),
                s.start_date.as_deref().and_then(date_field::parse_iso),
            ),
            None => (String::new(), String::new(), "monthly".into(), String::new(), String::new(), None),
        };
        let name_input = text_input_with("e.g. Netflix", &name, window, cx);
        name_input.update(cx, |s, cx| s.focus(window, cx));
        let amount_input = text_input_with("0.00", &amount, window, cx);
        let _subs = submit_on_enter(&[&name_input, &amount_input], Self::submit, cx);
        Self {
            db,
            editing: editing.map(|s| s.id),
            name: name_input,
            amount: amount_input,
            frequency: choice(frequency_items, Some(&frequency), window, cx),
            category: choice(category_items, Some(&category), window, cx),
            account: choice(account_items, Some(&account), window, cx),
            start_date: date_field::single(start, window, cx),
            _subs,
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let name = text_of(&self.name, cx);
        let amount_text = text_of(&self.amount, cx);
        if name.is_empty() || amount_text.is_empty() {
            return;
        }
        let Some(amount) = parse_cents(&amount_text) else {
            return cx.emit(FormEvent::Failed("Amount is not valid".into()));
        };
        let frequency = choice_value(&self.frequency, cx).unwrap_or_else(|| "monthly".into());
        let none_if_blank = |v: Option<String>| v.filter(|s| !s.is_empty());
        let category_id = none_if_blank(choice_value(&self.category, cx));
        let account_id = none_if_blank(choice_value(&self.account, cx));
        let start_date = date_field::picked(&self.start_date, cx).map(|d| d.format(date_field::FORMAT).to_string());

        let result = match &self.editing {
            None => commands::create_subscription(
                &self.db,
                CreateSubscriptionRequest {
                    name: name.clone(),
                    amount,
                    currency: "USD".into(),
                    frequency,
                    category_id,
                    account_id,
                    start_date,
                },
            )
            .map(|_| format!("Added subscription {name}")),
            Some(id) => commands::update_subscription(
                &self.db,
                UpdateSubscriptionRequest {
                    id: id.clone(),
                    name: Some(name.clone()),
                    amount: Some(amount),
                    currency: Some("USD".into()),
                    frequency: Some(frequency),
                    category_id: Some(category_id),
                    account_id: Some(account_id),
                    start_date: Some(start_date),
                    is_active: None,
                },
            )
            .map(|_| format!("Saved subscription {name}")),
        };
        match result {
            Ok(message) => cx.emit(FormEvent::Saved(message)),
            Err(e) => cx.emit(FormEvent::Failed(format!("Could not save subscription: {e}"))),
        }
    }
}

impl Render for SubscriptionForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.editing.is_some();
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(field("Name", input("sub-name", &self.name, window, cx), cx))
            .child(
                div()
                    .flex()
                    .gap(rems(0.5))
                    .child(field("Amount", input("sub-amount", &self.amount, window, cx), cx))
                    .child(field("Period", select("sub-frequency", &self.frequency, window, cx), cx)),
            )
            .child(field("Category", select("sub-category", &self.category, window, cx), cx))
            .child(field("Account", select("sub-account", &self.account, window, cx), cx))
            .child(field("Start Date", DatePicker::new(&self.start_date).placeholder("Pick a date").cleanable(true), cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(button("sub-cancel", "Cancel", ButtonVariant::Outline, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel))))
                    .child(button("sub-submit", if editing { "Save" } else { "Add" }, ButtonVariant::Primary, cx).on_click(cx.listener(|this, _, _, cx| this.submit(cx)))),
            );
        modal(
            if editing { "Edit Subscription" } else { "Add Subscription" },
            body,
            cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel)),
            cx,
        ).on_action(cx.listener(|_, _: &super::ModalCancel, _, cx| cx.emit(FormEvent::Cancel)))
    }
}

/// Green for an active subscription, yellow for an inactive one.
fn status_dot(active: bool, cx: &App) -> gpui_kit::Div {
    let t = cx.omarchy();
    div()
        .size(rems(0.625))
        .rounded_full()
        .bg(if active { t.success } else { t.warning })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use gpui_kit::component::calendar::Date;
    use gpui_kit::{TestAppContext, VisualTestContext};

    fn setup(cx: &mut TestAppContext) -> (Entity<SubscriptionsScreen>, Arc<DbState>, &mut VisualTestContext) {
        init(cx);
        let db = seeded_db();
        let handle = db.clone();
        let (screen, vcx) = cx.add_window_view(move |window, cx| SubscriptionsScreen::new(db, window, cx));
        (screen, handle, vcx)
    }

    fn make(db: &DbState, name: &str, amount: i64, frequency: &str) -> Subscription {
        commands::create_subscription(
            db,
            CreateSubscriptionRequest {
                name: name.into(),
                amount,
                currency: "USD".into(),
                frequency: frequency.into(),
                category_id: None,
                account_id: None,
                start_date: Some("2026-01-15".into()),
            },
        )
        .unwrap()
    }

    fn reload(screen: &Entity<SubscriptionsScreen>, cx: &mut VisualTestContext) {
        cx.update(|_, cx| screen.update(cx, |s, cx| s.reload(cx)));
    }

    fn type_into(input: &Entity<InputState>, text: &str, cx: &mut VisualTestContext) {
        cx.update(|window, cx| input.update(cx, |i, cx| i.set_value(text, window, cx)));
    }

    fn form_of(screen: &Entity<SubscriptionsScreen>, cx: &mut VisualTestContext) -> Entity<SubscriptionForm> {
        cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Form(f)) => f.clone(),
            _ => panic!("form should be open"),
        })
    }

    #[gpui_kit::test]
    fn add_subscription_with_start_date(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = form_of(&screen, cx);
        let (name, amount, date) = cx.update(|_, cx| {
            let f = form.read(cx);
            (f.name.clone(), f.amount.clone(), f.start_date.clone())
        });
        type_into(&name, "Netflix", cx);
        type_into(&amount, "15.99", cx);
        cx.update(|window, cx| {
            date.update(cx, |d, cx| d.set_date(Date::Single(chrono::NaiveDate::from_ymd_opt(2026, 3, 9)), window, cx))
        });
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_dialog());
            let added = s.subscriptions().iter().find(|x| x.name == "Netflix").expect("saved");
            assert_eq!(added.amount, 1599);
            assert_eq!(added.frequency, "monthly");
            assert_eq!(added.start_date.as_deref(), Some("2026-03-09"));
        });
    }

    #[gpui_kit::test]
    fn invalid_or_blank_input_keeps_form_open(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = form_of(&screen, cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| assert!(screen.read(cx).has_dialog(), "blank form is ignored"));
        let (name, amount) = cx.update(|_, cx| (form.read(cx).name.clone(), form.read(cx).amount.clone()));
        type_into(&name, "Bad", cx);
        type_into(&amount, "abc", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            assert!(screen.read(cx).has_dialog());
            assert!(screen.read(cx).subscriptions().is_empty());
        });
    }

    #[gpui_kit::test]
    fn edit_updates_and_can_clear_the_category(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let cat = commands::get_categories(&db).unwrap().into_iter().find(|c| c.category_type == "expense").unwrap();
        let created = commands::create_subscription(&db, CreateSubscriptionRequest {
            name: "Gym".into(), amount: 3000, currency: "USD".into(), frequency: "monthly".into(),
            category_id: Some(cat.id.clone()), account_id: None, start_date: None,
        }).unwrap();
        reload(&screen, cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_form(Some(created.clone()), window, cx)));
        let form = form_of(&screen, cx);
        let (amount, category) = cx.update(|_, cx| {
            let f = form.read(cx);
            assert_eq!(f.amount.read(cx).value().as_ref(), "30.00");
            assert_eq!(choice_value(&f.category, cx).as_deref(), Some(cat.id.as_str()));
            (f.amount.clone(), f.category.clone())
        });
        type_into(&amount, "35", cx);
        cx.update(|_, cx| category.update(cx, |c, cx| c.set_selected(Some(0), cx)));
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let saved = s.subscriptions().iter().find(|x| x.id == created.id).unwrap();
            assert_eq!(saved.amount, 3500);
            assert_eq!(saved.category_id, None, "choosing None clears the category");
            assert_eq!(s.subscriptions().len(), 1);
        });
    }

    #[gpui_kit::test]
    fn recording_a_payment_creates_an_expense(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let sub = make(&db, "Spotify", 999, "monthly");
        reload(&screen, cx);
        let before = commands::get_transactions(&db).unwrap().len();
        cx.update(|_, cx| screen.update(cx, |s, cx| s.pay(&sub, cx)));
        let after = commands::get_transactions(&db).unwrap();
        assert_eq!(after.len(), before + 1);
        let paid = after.iter().find(|t| t.description == "Spotify").expect("payment transaction");
        assert_eq!(paid.amount, 999);
        assert_eq!(paid.transaction_type, "expense");
    }

    #[gpui_kit::test]
    fn bulk_delete_goes_through_confirmation(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        for n in ["A", "B", "C"] {
            make(&db, n, 100, "monthly");
        }
        reload(&screen, cx);
        cx.update(|_, cx| screen.update(cx, |s, cx| s.toggle_page(cx)));
        cx.update(|_, cx| assert_eq!(screen.read(cx).selected.len(), 3));
        let ids: Vec<String> = cx.update(|_, cx| screen.read(cx).selected.iter().cloned().collect());
        cx.update(|_, cx| screen.update(cx, |s, cx| s.ask_delete(ids, cx)));

        // Cancelling keeps everything.
        let dialog = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Confirm(d, _)) => d.clone(),
            _ => panic!("confirm dialog should be open"),
        });
        cx.update(|_, cx| dialog.update(cx, |_, cx| cx.emit(ConfirmEvent::Cancel)));
        cx.update(|_, cx| {
            assert!(!screen.read(cx).has_dialog());
            assert_eq!(screen.read(cx).subscriptions().len(), 3);
        });

        // Confirming deletes them and clears the selection.
        let ids: Vec<String> = cx.update(|_, cx| screen.read(cx).selected.iter().cloned().collect());
        cx.update(|_, cx| screen.update(cx, |s, cx| s.ask_delete(ids, cx)));
        let dialog = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Confirm(d, _)) => d.clone(),
            _ => panic!("confirm dialog should be open"),
        });
        cx.update(|_, cx| dialog.update(cx, |_, cx| cx.emit(ConfirmEvent::Confirm)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(s.subscriptions().is_empty());
            assert!(s.selected.is_empty());
            assert!(!s.has_dialog());
        });
    }

    #[gpui_kit::test]
    fn frequency_filter_narrows_the_list_and_resets_the_page(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        make(&db, "M1", 1000, "monthly");
        make(&db, "M2", 500, "monthly");
        make(&db, "Y1", 12000, "yearly");
        reload(&screen, cx);
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.filtered().len(), 3);
            assert_eq!(total_amount(&s.filtered()), 13500);
        });
        cx.update(|_, cx| screen.update(cx, |s, cx| { s.page = 2; cx.notify(); }));
        let filter = cx.update(|_, cx| screen.read(cx).filter.clone());
        // Items: all, weekly, bi_weekly, monthly, ... so index 3 is monthly.
        cx.update(|_, cx| filter.update(cx, |f, cx| f.set_selected(Some(3), cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.applied_filter, "monthly");
            assert_eq!(s.page, 1, "changing the filter returns to page one");
            assert_eq!(s.filtered().len(), 2);
            assert_eq!(total_amount(&s.filtered()), 1500);
        });
    }

    #[gpui_kit::test]
    fn open_add_shows_the_form(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        cx.update(|_, cx| assert!(screen.read(cx).has_dialog()));
    }

    #[gpui_kit::test]
    fn escape_dismisses_the_delete_confirmation_without_deleting(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let sub = make(&db, "Keep me", 100, "monthly");
        reload(&screen, cx);
        cx.update(|_, cx| screen.update(cx, |s, cx| s.ask_delete(vec![sub.id.clone()], cx)));
        cx.update(|_, cx| assert!(screen.read(cx).has_dialog()));
        // The dialog takes focus while drawing its first frame; a person presses
        // a key some frames later.
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("escape");
        cx.update(|_, cx| {
            assert!(!screen.read(cx).has_dialog());
            assert_eq!(screen.read(cx).subscriptions().len(), 1);
        });
    }
}
