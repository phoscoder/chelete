use super::accounts::FormEvent;
use super::csv_import::{CsvImportDialog, CsvImportEvent};
use super::date_field;
use super::date_filter_control::DateFilterControl;
use super::icons::category_icon;
use super::widgets::*;
use chelete_lib::commands::{
    self, Account, Category, CreateTransactionRequest, ExportData, Transaction, UpdateTransactionRequest,
};
use chelete_lib::database::DbState;
use chelete_lib::date_filter::is_date_in_range;
use chelete_lib::export::ExportFormat;
use chelete_lib::format::{format_money, parse_cents};
use chelete_lib::paging::{page_slice, Pager};
use gpui_kit::base::input::InputState;
use gpui_kit::component::date_picker::DatePicker;
use gpui_kit::component::date_picker::DatePickerState;
use gpui_kit::{
    div, prelude::*, px, rems, Context, Entity, EventEmitter, IntoElement, Render, SharedString,
    Subscription, Window,
};
use gpui_omarchy::{
    button, icon, icon_button, input, select, ActiveTheme as _, ButtonVariant, ChoiceState, IconName,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

enum Dialog {
    Form(Entity<TransactionForm>),
    View(Transaction),
    Confirm(Entity<ConfirmDialog>, Vec<String>),
    Export(Entity<ExportDialog>),
    Import(Entity<CsvImportDialog>),
}

pub struct TransactionsScreen {
    db: Arc<DbState>,
    transactions: Vec<Transaction>,
    accounts: Vec<Account>,
    categories: Vec<Category>,
    date_filter: DateFilterControl,
    per_page: Entity<ChoiceState>,
    applied_per_page: usize,
    page: usize,
    selected: HashSet<String>,
    dialog: Option<Dialog>,
    mappings_path: PathBuf,
    _dialog_sub: Option<Subscription>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<ScreenEvent> for TransactionsScreen {}

impl TransactionsScreen {
    pub fn new(db: Arc<DbState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let date_filter = DateFilterControl::new(window, cx);
        let per_page = per_page_choice(25, window, cx);
        let mut subs = date_filter.observe(cx, |this: &mut Self, cx| {
            this.page = 1;
            cx.notify();
        });
        subs.push(cx.observe(&per_page, |this, _, cx| {
            let n = choice_value(&this.per_page, cx).and_then(|v| v.parse().ok()).unwrap_or(this.applied_per_page);
            if n != this.applied_per_page {
                this.applied_per_page = n;
                this.page = 1;
            }
            cx.notify();
        }));
        let mut screen = Self {
            db,
            transactions: Vec::new(),
            accounts: Vec::new(),
            categories: Vec::new(),
            date_filter,
            per_page,
            applied_per_page: 25,
            page: 1,
            selected: HashSet::new(),
            dialog: None,
            mappings_path: chelete_lib::import::session::SavedMappings::default_path(),
            _dialog_sub: None,
            _subs: subs,
        };
        screen.load();
        screen
    }

    fn load(&mut self) {
        self.transactions = commands::get_transactions(&self.db).unwrap_or_default();
        self.accounts = commands::get_accounts(&self.db).unwrap_or_default();
        self.categories = commands::get_categories(&self.db).unwrap_or_default();
        let live: HashSet<&String> = self.transactions.iter().map(|t| &t.id).collect();
        self.selected.retain(|id| live.contains(id));
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load();
        cx.notify();
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    fn filtered(&self, cx: &gpui_kit::App) -> Vec<&Transaction> {
        let (start, end) = self.date_filter.bounds(chrono::Local::now().date_naive(), cx);
        self.transactions
            .iter()
            .filter(|t| is_date_in_range(&t.transaction_date, &start, &end))
            .collect()
    }

    fn pager(&self, cx: &gpui_kit::App) -> Pager {
        Pager::new(self.page, self.applied_per_page, self.filtered(cx).len())
    }

    #[cfg(test)]
    fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn open_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_form(None, window, cx);
    }

    fn open_form(&mut self, editing: Option<Transaction>, window: &mut Window, cx: &mut Context<Self>) {
        let form = cx.new(|cx| TransactionForm::new(self.db.clone(), &self.accounts, &self.categories, editing, window, cx));
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
        let noun = plural(ids.len(), "transaction");
        let plural_s = if ids.len() == 1 { "" } else { "s" };
        let dialog = cx.new(|_| {
            ConfirmDialog::new(
                format!("Delete {noun}?"),
                format!("This will permanently remove the selected transaction{plural_s} and adjust account balances. This action cannot be undone."),
                "Delete",
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
        match commands::delete_transactions(&self.db, ids.clone()) {
            Ok(()) => {
                for id in &ids {
                    self.selected.remove(id);
                }
                self.load();
                self.page = self.pager(cx).page;
                cx.emit(ScreenEvent::ok(format!("Deleted {}", plural(ids.len(), "transaction"))));
            }
            Err(e) => cx.emit(ScreenEvent::error(format!("Could not delete: {e}"))),
        }
    }

    /// Open the CSV import dialog and ask for a file straight away.
    pub fn open_import(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dialog = self.new_import_dialog(window, cx);
        dialog.update(cx, |d, cx| d.choose_file(window, cx));
    }

    fn new_import_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<CsvImportDialog> {
        let dialog = cx.new(|cx| {
            CsvImportDialog::new(self.db.clone(), self.accounts.clone(), self.categories.clone(), self.mappings_path.clone(), window, cx)
        });
        self._dialog_sub = Some(cx.subscribe(&dialog, |this, _, event: &CsvImportEvent, cx| {
            if matches!(event, CsvImportEvent::Done) {
                this.load();
            }
            this.close_dialog(cx);
        }));
        self.dialog = Some(Dialog::Import(dialog.clone()));
        cx.notify();
        dialog
    }

    pub fn open_export(&mut self, cx: &mut Context<Self>) {
        let data = match commands::export_data(&self.db) {
            Ok(d) => d,
            Err(e) => return cx.emit(ScreenEvent::error(format!("Could not read data: {e}"))),
        };
        let dialog = cx.new(|_| ExportDialog::new(data));
        self._dialog_sub = Some(cx.subscribe(&dialog, |this, _, event: &ExportEvent, cx| {
            match event {
                ExportEvent::Cancel => {}
                ExportEvent::Written(path) => cx.emit(ScreenEvent::ok(format!("Exported to {}", path.display()))),
                ExportEvent::Failed(message) => {
                    cx.emit(ScreenEvent::error(message.clone()));
                    return;
                }
            }
            this.close_dialog(cx);
        }));
        self.dialog = Some(Dialog::Export(dialog));
        cx.notify();
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.selected.remove(id) {
            self.selected.insert(id.to_string());
        }
        cx.notify();
    }

    fn toggle_page(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered(cx);
        let ids: Vec<String> = page_slice(&filtered, &self.pager(cx)).iter().map(|t| t.id.clone()).collect();
        let all = !ids.is_empty() && ids.iter().all(|id| self.selected.contains(id));
        for id in ids {
            if all {
                self.selected.remove(&id);
            } else {
                self.selected.insert(id);
            }
        }
        cx.notify();
    }

    fn category(&self, id: &Option<String>) -> Option<&Category> {
        id.as_ref().and_then(|id| self.categories.iter().find(|c| &c.id == id))
    }

    fn account_name(&self, id: &str) -> Option<&str> {
        self.accounts.iter().find(|a| a.id == id).map(|a| a.name.as_str())
    }
}

fn signed(t: &Transaction) -> i64 {
    if t.transaction_type == "expense" {
        -t.amount
    } else {
        t.amount
    }
}

fn short_date(iso: &str) -> String {
    date_field::parse_iso(iso)
        .map(|d| d.format("%b %e").to_string())
        .unwrap_or_else(|| iso.to_string())
}

impl Render for TransactionsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let today = chrono::Local::now().date_naive();
        let filtered = self.filtered(cx);
        let pager = self.pager(cx);
        let rows: Vec<Transaction> = page_slice(&filtered, &pager).iter().map(|t| (*t).clone()).collect();
        let none_at_all = self.transactions.is_empty();
        let selected_on_page = rows.iter().filter(|r| self.selected.contains(&r.id)).count();
        let header_state = selection_state(selected_on_page, rows.len());
        drop(filtered);

        let actions = if self.selected.is_empty() {
            div()
                .flex()
                .gap(rems(0.5))
                .child(
                    button("import-csv", "Import CSV", ButtonVariant::Outline, cx)
                        .child(icon(IconName::Upload).size(rems(0.875)))
                        .on_click(cx.listener(|this, _, window, cx| this.open_import(window, cx))),
                )
                .child(
                    button("export-data", "Export Data", ButtonVariant::Outline, cx)
                        .child(icon(IconName::Download).size(rems(0.875)))
                        .on_click(cx.listener(|this, _, _, cx| this.open_export(cx))),
                )
                .child(
                    button("add-transaction", "+ Add", ButtonVariant::Primary, cx)
                        .on_click(cx.listener(|this, _, window, cx| this.open_add(window, cx))),
                )
                .into_any_element()
        } else {
            let ids: Vec<String> = self.selected.iter().cloned().collect();
            button("delete-selected", format!("Delete {} selected", self.selected.len()), ButtonVariant::Danger, cx)
                .on_click(cx.listener(move |this, _, _, cx| this.ask_delete(ids.clone(), cx)))
                .into_any_element()
        };

        let body = if rows.is_empty() {
            let text = if none_at_all { "No transactions yet. Add one to get started." } else { "No transactions match the selected filters." };
            div().py(rems(3.)).flex().justify_center().text_color(t.secondary).child(text).into_any_element()
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
                .child(cell(Some(6.), "Date"))
                .child(cell(None, "Description"))
                .child(cell(Some(6.), "Type"))
                .child(cell(None, "Category"))
                .child(cell(None, "Account"))
                .child(cell(Some(8.), div().flex().justify_end().child("Amount")))
                .child(cell(Some(7.5), ""));
            let mut list = div().flex().flex_col().border_1().border_color(t.border).child(header);
            for tx in rows {
                let category = self.category(&tx.category_id).cloned();
                let account = self.account_name(&tx.account_id).map(str::to_string);
                let checked = self.selected.contains(&tx.id);
                let row_id = tx.id.clone();
                let key = |prefix: &str| SharedString::from(format!("{prefix}-{}", tx.id));
                let (view, edit, del) = (tx.clone(), tx.clone(), tx.id.clone());
                let tint = if tx.transaction_type == "income" { t.success } else { t.danger };
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
                        .child(cell(Some(6.), short_date(&tx.transaction_date)))
                        .child(cell(None, tx.description.clone()))
                        .child(cell(Some(6.), div().text_color(tint).child(tx.transaction_type.clone())))
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
                        .child(cell(Some(8.), div().flex().justify_end().text_color(tint).child(format_money(signed(&tx)))))
                        .child(cell(
                            Some(7.5),
                            div()
                                .flex()
                                .justify_end()
                                .gap(rems(0.125))
                                .child(icon_button(key("view"), IconName::Eye, "View transaction", ButtonVariant::Secondary, cx)
                                    .on_click(cx.listener(move |this, _, _, cx| { this.dialog = Some(Dialog::View(view.clone())); cx.notify(); })))
                                .child(icon_button(key("edit"), IconName::Pencil, "Edit transaction", ButtonVariant::Secondary, cx)
                                    .on_click(cx.listener(move |this, _, window, cx| this.open_form(Some(edit.clone()), window, cx))))
                                .child(icon_button(key("del"), IconName::Trash, "Delete transaction", ButtonVariant::Danger, cx)
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
            Dialog::Export(e) => e.clone().into_any_element(),
            Dialog::Import(i) => i.clone().into_any_element(),
            Dialog::View(tx) => self.view_dialog(tx, cx).into_any_element(),
        });
        let filter_row = self.date_filter.render(today, window, cx);

        div()
            .id("transactions")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .p(rems(1.5))
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(page_header("Transactions", actions, cx))
            .child(filter_row)
            .child(body)
            .children(dialog)
    }
}

impl TransactionsScreen {
    fn view_dialog(&self, tx: &Transaction, cx: &mut Context<Self>) -> impl IntoElement {
        let category = self.category(&tx.category_id).map(|c| c.name.clone());
        let account = self.account_name(&tx.account_id).map(str::to_string);
        let row = |label: &str, value: String| div().flex().justify_between().child(dim(label.to_string(), cx)).child(value);
        let mut body = div()
            .flex()
            .flex_col()
            .gap(rems(0.5))
            .child(row("Date", short_date(&tx.transaction_date)))
            .child(row("Description", tx.description.clone()));
        if let Some(m) = &tx.merchant {
            body = body.child(row("Merchant", m.clone()));
        }
        body = body
            .child(row("Type", tx.transaction_type.clone()))
            .child(row("Amount", format_money(signed(tx))))
            .child(row("Account", account.unwrap_or_else(|| "—".into())))
            .child(row("Category", category.unwrap_or_else(|| "—".into())));
        if let Some(n) = &tx.notes {
            body = body.child(row("Notes", n.clone()));
        }
        let body = body.child(
            div().flex().justify_end().pt(rems(0.5)).child(
                button("view-close", "Close", ButtonVariant::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.close_dialog(cx))),
            ),
        );
        modal("Transaction", body, cx.listener(|this, _, _, cx| this.close_dialog(cx)), cx)
    }
}

// ── add / edit ──────────────────────────────────────────────────────

pub struct TransactionForm {
    db: Arc<DbState>,
    categories: Vec<Category>,
    editing: Option<Transaction>,
    kind: String,
    amount: Entity<InputState>,
    description: Entity<InputState>,
    account: Entity<ChoiceState>,
    category: Entity<ChoiceState>,
    date: Entity<DatePickerState>,
}

impl EventEmitter<FormEvent> for TransactionForm {}

fn category_items(categories: &[Category], kind: &str) -> Vec<(String, String)> {
    let mut items = vec![(String::new(), "None".to_string())];
    items.extend(categories.iter().filter(|c| c.category_type == kind).map(|c| (c.id.clone(), c.name.clone())));
    items
}

impl TransactionForm {
    fn new(
        db: Arc<DbState>,
        accounts: &[Account],
        categories: &[Category],
        editing: Option<Transaction>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let kind = editing.as_ref().map(|t| t.transaction_type.clone()).unwrap_or_else(|| "expense".into());
        let amount = editing.as_ref().map(|t| format!("{:.2}", t.amount as f64 / 100.0)).unwrap_or_default();
        let description = editing.as_ref().map(|t| t.description.clone()).unwrap_or_default();
        let account_id = editing.as_ref().map(|t| t.account_id.clone()).or_else(|| accounts.first().map(|a| a.id.clone()));
        let category_id = editing.as_ref().and_then(|t| t.category_id.clone()).unwrap_or_default();
        let date = editing
            .as_ref()
            .and_then(|t| date_field::parse_iso(&t.transaction_date))
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        let amount_input = text_input_with("0.00", &amount, window, cx);
        amount_input.update(cx, |s, cx| s.focus(window, cx));
        Self {
            db,
            categories: categories.to_vec(),
            kind: kind.clone(),
            amount: amount_input,
            description: text_input_with("e.g. Groceries", &description, window, cx),
            account: choice(accounts.iter().map(|a| (a.id.clone(), a.name.clone())).collect(), account_id.as_deref(), window, cx),
            category: choice(category_items(categories, &kind), Some(&category_id), window, cx),
            date: date_field::single(Some(date), window, cx),
            editing,
        }
    }

    fn set_kind(&mut self, kind: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.kind == kind {
            return;
        }
        self.kind = kind.to_string();
        // Categories belong to one type, so the list is rebuilt on a switch.
        self.category = choice(category_items(&self.categories, kind), Some(""), window, cx);
        cx.notify();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let amount_text = text_of(&self.amount, cx);
        let cents = match parse_cents(&amount_text) {
            Some(c) if c > 0 => c,
            _ => return,
        };
        let Some(account_id) = choice_value(&self.account, cx) else {
            return cx.emit(FormEvent::Failed("Add an account first".into()));
        };
        let category_id = choice_value(&self.category, cx).filter(|c| !c.is_empty());
        let description = text_of(&self.description, cx);
        let date = date_field::picked(&self.date, cx)
            .unwrap_or_else(|| chrono::Local::now().date_naive())
            .format(date_field::FORMAT)
            .to_string();

        let result = match &self.editing {
            None => commands::create_transaction(
                &self.db,
                CreateTransactionRequest {
                    account_id,
                    category_id,
                    transaction_type: self.kind.clone(),
                    amount: cents,
                    currency: "USD".into(),
                    description,
                    merchant: None,
                    notes: None,
                    transaction_date: date,
                },
            )
            .map(|_| "Transaction added".to_string()),
            Some(existing) => commands::update_transaction(
                &self.db,
                UpdateTransactionRequest {
                    id: existing.id.clone(),
                    account_id: Some(account_id),
                    category_id: Some(category_id),
                    transaction_type: Some(self.kind.clone()),
                    amount: Some(cents),
                    currency: Some("USD".into()),
                    description: Some(description),
                    merchant: None,
                    notes: None,
                    transaction_date: Some(date),
                },
            )
            .map(|_| "Transaction saved".to_string()),
        };
        match result {
            Ok(message) => cx.emit(FormEvent::Saved(message)),
            Err(e) => cx.emit(FormEvent::Failed(format!("Could not save transaction: {e}"))),
        }
    }
}

impl Render for TransactionForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.editing.is_some();
        let kind_button = |id: &'static str, label: &'static str, value: &'static str, cx: &mut Context<Self>| {
            button(id, label, if self.kind == value { ButtonVariant::Primary } else { ButtonVariant::Outline }, cx)
                .flex_1()
                .justify_center()
                .on_click(cx.listener(move |this, _, window, cx| this.set_kind(value, window, cx)))
        };
        let toggle = div()
            .flex()
            .gap(rems(0.5))
            .child(kind_button("kind-expense", "Expense", "expense", cx))
            .child(kind_button("kind-income", "Income", "income", cx));
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(field("Type", toggle, cx))
            .child(field("Amount", input("tx-amount", &self.amount, window, cx), cx))
            .child(field("Description", input("tx-description", &self.description, window, cx), cx))
            .child(
                div()
                    .flex()
                    .gap(rems(0.5))
                    .child(field("Account", select("tx-account", &self.account, window, cx), cx))
                    .child(field("Category", select("tx-category", &self.category, window, cx), cx)),
            )
            .child(field("Date", DatePicker::new(&self.date).placeholder("Pick a date"), cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(button("tx-cancel", "Cancel", ButtonVariant::Outline, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel))))
                    .child(button("tx-submit", if editing { "Save" } else { "Add" }, ButtonVariant::Primary, cx).on_click(cx.listener(|this, _, _, cx| this.submit(cx)))),
            );
        modal(
            if editing { "Edit Transaction" } else { "Add Transaction" },
            body,
            cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel)),
            cx,
        )
    }
}

// ── export ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ExportEvent {
    Cancel,
    Written(PathBuf),
    Failed(String),
}

pub struct ExportDialog {
    data: ExportData,
    format: ExportFormat,
}

impl EventEmitter<ExportEvent> for ExportDialog {}

impl ExportDialog {
    fn new(data: ExportData) -> Self {
        Self { data, format: ExportFormat::Csv }
    }

    /// Render the chosen format and write it to `path`.
    pub fn write_to(&self, path: &std::path::Path) -> Result<(), String> {
        let contents = self.format.contents(&self.data)?;
        commands::write_export_file(path.to_string_lossy().to_string(), contents)
    }

    fn choose_path_and_write(&mut self, cx: &mut Context<Self>) {
        let directory = dirs::download_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from("."));
        let picked = cx.prompt_for_new_path(&directory, Some(self.format.default_file_name()));
        cx.spawn(async move |this, cx| {
            let outcome = match picked.await {
                Ok(Ok(Some(path))) => this.update(cx, |this, cx| match this.write_to(&path) {
                    Ok(()) => cx.emit(ExportEvent::Written(path)),
                    Err(e) => cx.emit(ExportEvent::Failed(format!("Export failed: {e}"))),
                }),
                Ok(Ok(None)) | Err(_) => Ok(()),
                Ok(Err(e)) => this.update(cx, |_, cx| cx.emit(ExportEvent::Failed(format!("Export failed: {e}")))),
            };
            outcome.ok();
        })
        .detach();
    }
}

impl Render for ExportDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let format_button = |id: &'static str, label: &'static str, format: ExportFormat, cx: &mut Context<Self>| {
            button(id, label, if self.format == format { ButtonVariant::Primary } else { ButtonVariant::Outline }, cx)
                .flex_1()
                .justify_center()
                .on_click(cx.listener(move |this, _, _, cx| { this.format = format; cx.notify(); }))
        };
        let summary = match self.format {
            ExportFormat::Csv => format!("{} transactions, one row each.", self.data.transactions.len()),
            ExportFormat::Json => "Accounts, categories, transactions and subscriptions.".to_string(),
        };
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(field(
                "Format",
                div().flex().gap(rems(0.5)).child(format_button("fmt-csv", "CSV", ExportFormat::Csv, cx)).child(format_button("fmt-json", "JSON", ExportFormat::Json, cx)),
                cx,
            ))
            .child(dim(summary, cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(button("export-cancel", "Cancel", ButtonVariant::Outline, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(ExportEvent::Cancel))))
                    .child(button("export-go", "Export", ButtonVariant::Primary, cx).on_click(cx.listener(|this, _, _, cx| this.choose_path_and_write(cx)))),
            );
        modal("Export Data", body, cx.listener(|_, _, _, cx| cx.emit(ExportEvent::Cancel)), cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use chelete_lib::date_filter::DateFilter;
    use gpui_kit::component::calendar::Date;
    use gpui_kit::{TestAppContext, VisualTestContext};

    fn setup(cx: &mut TestAppContext) -> (Entity<TransactionsScreen>, Arc<DbState>, &mut VisualTestContext) {
        init(cx);
        let db = seeded_db();
        let handle = db.clone();
        let (screen, vcx) = cx.add_window_view(move |window, cx| TransactionsScreen::new(db, window, cx));
        (screen, handle, vcx)
    }

    fn type_into(input: &Entity<InputState>, text: &str, cx: &mut VisualTestContext) {
        cx.update(|window, cx| input.update(cx, |i, cx| i.set_value(text, window, cx)));
    }

    fn form_of(screen: &Entity<TransactionsScreen>, cx: &mut VisualTestContext) -> Entity<TransactionForm> {
        cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Form(f)) => f.clone(),
            _ => panic!("form should be open"),
        })
    }

    fn select_filter(screen: &Entity<TransactionsScreen>, filter: DateFilter, cx: &mut VisualTestContext) {
        let index = DateFilter::ALL_FILTERS.iter().position(|f| *f == filter).unwrap();
        let preset = cx.update(|_, cx| screen.read(cx).date_filter.preset.clone());
        cx.update(|_, cx| preset.update(cx, |c, cx| c.set_selected(Some(index), cx)));
    }

    #[gpui_kit::test]
    fn lists_all_seeded_transactions(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.transactions().len(), 25);
            assert_eq!(s.filtered(cx).len(), 25, "All Time shows everything");
            assert_eq!(s.pager(cx).label(), "1–25 of 25");
        });
    }

    #[gpui_kit::test]
    fn add_expense_updates_the_account_balance(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let account = commands::get_accounts(&db).unwrap().remove(0);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = form_of(&screen, cx);
        let (amount, description) = cx.update(|_, cx| (form.read(cx).amount.clone(), form.read(cx).description.clone()));
        type_into(&amount, "42.50", cx);
        type_into(&description, "Test purchase", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_dialog());
            assert_eq!(s.transactions().len(), 26);
            let added = s.transactions().iter().find(|t| t.description == "Test purchase").expect("saved");
            assert_eq!(added.amount, 4250);
            assert_eq!(added.transaction_type, "expense");
            assert_eq!(added.account_id, account.id, "defaults to the first account");
        });
        let after = commands::get_accounts(&db).unwrap().remove(0);
        assert_eq!(after.balance, account.balance - 4250);
    }

    #[gpui_kit::test]
    fn zero_or_blank_amount_is_ignored(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = form_of(&screen, cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        let amount = cx.update(|_, cx| form.read(cx).amount.clone());
        type_into(&amount, "0", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            assert!(screen.read(cx).has_dialog());
            assert_eq!(screen.read(cx).transactions().len(), 25);
        });
    }

    #[gpui_kit::test]
    fn switching_to_income_swaps_the_category_list(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = form_of(&screen, cx);
        let expense_names: Vec<String> = commands::get_categories(&db).unwrap().into_iter().filter(|c| c.category_type == "expense").map(|c| c.id).collect();
        cx.update(|window, cx| form.update(cx, |f, cx| f.set_kind("income", window, cx)));
        let (amount, description, category) = cx.update(|_, cx| {
            let f = form.read(cx);
            (f.amount.clone(), f.description.clone(), f.category.clone())
        });
        // Pick the first real income category (index 0 is "None").
        cx.update(|_, cx| category.update(cx, |c, cx| c.set_selected(Some(1), cx)));
        type_into(&amount, "100", cx);
        type_into(&description, "Paycheck", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let added = screen.read(cx).transactions().iter().find(|t| t.description == "Paycheck").cloned().expect("saved");
            assert_eq!(added.transaction_type, "income");
            let cat = added.category_id.expect("income category chosen");
            assert!(!expense_names.contains(&cat), "income form must not offer expense categories");
        });
    }

    #[gpui_kit::test]
    fn edit_changes_the_transaction_in_place(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        let original = cx.update(|_, cx| screen.read(cx).transactions()[0].clone());
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_form(Some(original.clone()), window, cx)));
        let form = form_of(&screen, cx);
        let (amount, date) = cx.update(|_, cx| {
            let f = form.read(cx);
            assert_eq!(f.amount.read(cx).value().as_ref(), format!("{:.2}", original.amount as f64 / 100.0));
            (f.amount.clone(), f.date.clone())
        });
        type_into(&amount, "1.23", cx);
        cx.update(|window, cx| date.update(cx, |d, cx| d.set_date(Date::Single(chrono::NaiveDate::from_ymd_opt(2026, 1, 2)), window, cx)));
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.transactions().len(), 25, "no duplicate");
            let saved = s.transactions().iter().find(|t| t.id == original.id).unwrap();
            assert_eq!(saved.amount, 123);
            assert_eq!(saved.transaction_date, "2026-01-02");
        });
    }

    #[gpui_kit::test]
    fn date_presets_filter_the_list(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        select_filter(&screen, DateFilter::Today, cx);
        let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(s.filtered(cx).iter().all(|t| t.transaction_date == today));
            assert!(s.filtered(cx).len() < 25);
        });
        select_filter(&screen, DateFilter::LastYear, cx);
        cx.update(|_, cx| {
            let s = screen.read(cx);
            let year = (chrono::Local::now().date_naive().format("%Y").to_string().parse::<i32>().unwrap() - 1).to_string();
            assert!(s.filtered(cx).iter().all(|t| t.transaction_date.starts_with(&year)));
        });
    }

    #[gpui_kit::test]
    fn custom_range_uses_the_range_picker(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        let all = cx.update(|_, cx| screen.read(cx).transactions().to_vec());
        let mut dates: Vec<_> = all.iter().map(|t| t.transaction_date.clone()).collect();
        dates.sort();
        let target = dates[dates.len() / 2].clone();
        let target_date = date_field::parse_iso(&target).unwrap();
        select_filter(&screen, DateFilter::Custom, cx);
        let range = cx.update(|_, cx| screen.read(cx).date_filter.range.clone());
        cx.update(|window, cx| range.update(cx, |r, cx| r.set_date(Date::Range(Some(target_date), Some(target_date)), window, cx)));
        cx.update(|_, cx| {
            let shown = screen.read(cx).filtered(cx);
            assert!(!shown.is_empty());
            assert!(shown.iter().all(|t| t.transaction_date == target));
            assert_eq!(shown.len(), all.iter().filter(|t| t.transaction_date == target).count());
        });
    }

    #[gpui_kit::test]
    fn changing_the_filter_resets_to_page_one(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|_, cx| screen.update(cx, |s, cx| { s.page = 3; cx.notify(); }));
        select_filter(&screen, DateFilter::ThisYear, cx);
        cx.update(|_, cx| assert_eq!(screen.read(cx).page, 1));
    }

    #[gpui_kit::test]
    fn per_page_paginates_the_list(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        let per_page = cx.update(|_, cx| screen.read(cx).per_page.clone());
        cx.update(|_, cx| per_page.update(cx, |c, cx| c.set_selected(Some(0), cx))); // 10 per page
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.applied_per_page, 10);
            assert_eq!(s.pager(cx).total_pages(), 3);
            assert_eq!(s.pager(cx).label(), "1–10 of 25");
        });
        cx.update(|_, cx| screen.update(cx, |s, cx| { s.page = 3; cx.notify(); }));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(page_slice(&s.filtered(cx), &s.pager(cx)).len(), 5);
        });
    }

    #[gpui_kit::test]
    fn bulk_delete_removes_transactions_and_restores_balances(cx: &mut TestAppContext) {
        let (screen, db, cx) = setup(cx);
        let before: i64 = commands::get_accounts(&db).unwrap().iter().map(|a| a.balance).sum();
        let victim = cx.update(|_, cx| screen.read(cx).transactions()[0].clone());
        cx.update(|_, cx| screen.update(cx, |s, cx| s.toggle(&victim.id, cx)));
        let ids: Vec<String> = cx.update(|_, cx| screen.read(cx).selected.iter().cloned().collect());
        assert_eq!(ids.len(), 1);
        cx.update(|_, cx| screen.update(cx, |s, cx| s.ask_delete(ids, cx)));
        let dialog = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Confirm(d, _)) => d.clone(),
            _ => panic!("confirm dialog should be open"),
        });
        cx.update(|_, cx| dialog.update(cx, |_, cx| cx.emit(ConfirmEvent::Confirm)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.transactions().len(), 24);
            assert!(s.selected.is_empty());
        });
        let after: i64 = commands::get_accounts(&db).unwrap().iter().map(|a| a.balance).sum();
        let delta = if victim.transaction_type == "expense" { victim.amount } else { -victim.amount };
        assert_eq!(after, before + delta, "deleting reverses the balance effect");
    }

    #[gpui_kit::test]
    fn export_writes_csv_and_json_files(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|_, cx| screen.update(cx, |s, cx| s.open_export(cx)));
        let dialog = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Export(d)) => d.clone(),
            _ => panic!("export dialog should be open"),
        });
        let dir = std::env::temp_dir().join(format!("chelete-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let csv_path = dir.join("out.csv");
        cx.update(|_, cx| dialog.read(cx).write_to(&csv_path)).unwrap();
        let csv = std::fs::read_to_string(&csv_path).unwrap();
        assert_eq!(csv.lines().count(), 26, "header plus 25 transactions");
        assert!(csv.starts_with("id,transaction_date,"));

        cx.update(|_, cx| dialog.update(cx, |d, _| d.format = ExportFormat::Json));
        let json_path = dir.join("out.json");
        cx.update(|_, cx| dialog.read(cx).write_to(&json_path)).unwrap();
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
        assert_eq!(json["transactions"].as_array().unwrap().len(), 25);
        assert_eq!(json["accounts"].as_array().unwrap().len(), 4);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[gpui_kit::test]
    fn export_to_an_unwritable_path_reports_an_error(cx: &mut TestAppContext) {
        let (screen, _db, cx) = setup(cx);
        cx.update(|_, cx| screen.update(cx, |s, cx| s.open_export(cx)));
        let dialog = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Export(d)) => d.clone(),
            _ => panic!("export dialog should be open"),
        });
        let result = cx.update(|_, cx| dialog.read(cx).write_to(std::path::Path::new("/nonexistent-dir/x/out.csv")));
        assert!(result.is_err());
    }
}
