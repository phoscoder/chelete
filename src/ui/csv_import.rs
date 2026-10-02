use super::widgets::*;
use chelete_lib::commands::{self, Account, Category, ImportOptions};
use chelete_lib::database::DbState;
use chelete_lib::format::format_money;
use chelete_lib::import::session::{
    flag_duplicates, is_valid_for_preview, set_column, type_label, valid_count, SavedMappings,
};
use chelete_lib::import::{parse_csv_preview, CsvField, CsvMapping, CsvPreview, ImportResult};
use gpui_kit::base::input::InputState;
use gpui_kit::base::CheckboxState;
use gpui_kit::{
    div, prelude::*, rems, Context, Entity, EventEmitter, IntoElement, PathPromptOptions, Render,
    Subscription, Window,
};
use gpui_omarchy::{alert, button, input, select, ActiveTheme as _, ButtonVariant, ChoiceState, Status};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Mapping,
    Preview,
    Result,
}

#[derive(Clone, Debug)]
pub enum CsvImportEvent {
    /// Closed after an import ran; the list should reload.
    Done,
    Cancel,
}

fn field_options() -> Vec<(String, String)> {
    let mut items = vec![(String::new(), "— not mapped —".to_string())];
    items.extend(
        CsvField::all()
            .iter()
            .map(|f| (field_value(*f).to_string(), f.label().to_string())),
    );
    items
}

fn field_value(field: CsvField) -> &'static str {
    match field {
        CsvField::Date => "date",
        CsvField::Description => "description",
        CsvField::Merchant => "merchant",
        CsvField::Amount => "amount",
        CsvField::IncomeAmount => "income_amount",
        CsvField::ExpenseAmount => "expense_amount",
        CsvField::Type => "type",
        CsvField::Category => "category",
        CsvField::Account => "account",
        CsvField::Currency => "currency",
        CsvField::Notes => "notes",
        CsvField::Ignore => "ignore",
    }
}

fn field_from_value(value: &str) -> Option<CsvField> {
    CsvField::all().iter().copied().find(|f| field_value(*f) == value)
}

pub struct CsvImportDialog {
    db: Arc<DbState>,
    stage: Stage,
    path: Option<String>,
    mapping: CsvMapping,
    preview: Option<CsvPreview>,
    duplicates: Vec<bool>,
    result: Option<ImportResult>,
    error: Option<String>,
    skip_duplicates: bool,
    column_choices: Vec<(String, Entity<ChoiceState>)>,
    account: Entity<ChoiceState>,
    category: Entity<ChoiceState>,
    default_type: Entity<ChoiceState>,
    debit_alias: Entity<ChoiceState>,
    credit_alias: Entity<ChoiceState>,
    currency: Entity<InputState>,
    accounts: Vec<Account>,
    categories: Vec<Category>,
    saved: SavedMappings,
    saved_path: PathBuf,
    syncing: bool,
    focus: ModalFocus,
    _subs: Vec<Subscription>,
}

impl EventEmitter<CsvImportEvent> for CsvImportDialog {}

impl CsvImportDialog {
    pub fn new(
        db: Arc<DbState>,
        accounts: Vec<Account>,
        categories: Vec<Category>,
        saved_path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut account_items = vec![(String::new(), "None".to_string())];
        account_items.extend(accounts.iter().map(|a| (a.id.clone(), a.name.clone())));
        let mut category_items = vec![(String::new(), "None".to_string())];
        category_items.extend(categories.iter().map(|c| (c.id.clone(), format!("{} ({})", c.name, c.category_type))));
        let type_items = || vec![("expense".to_string(), "Expense".to_string()), ("income".to_string(), "Income".to_string())];
        let account = choice(account_items, Some(""), window, cx);
        let category = choice(category_items, Some(""), window, cx);
        let default_type = choice(
            vec![(String::new(), "None".into()), ("expense".into(), "Expense".into()), ("income".into(), "Income".into())],
            Some(""),
            window,
            cx,
        );
        let debit_alias = choice(type_items(), Some("expense"), window, cx);
        let credit_alias = choice(type_items(), Some("income"), window, cx);
        let currency = text_input_with("USD", "USD", window, cx);
        let mut subs = Vec::new();
        for c in [&account, &category, &default_type, &debit_alias, &credit_alias] {
            subs.push(cx.observe(c, |this, _, cx| this.read_controls(cx)));
        }
        subs.push(cx.observe(&currency, |this, _, cx| this.read_controls(cx)));
        Self {
            saved: SavedMappings::load_from(&saved_path),
            saved_path,
            db,
            stage: Stage::Mapping,
            path: None,
            mapping: CsvMapping::default(),
            preview: None,
            duplicates: Vec::new(),
            result: None,
            error: None,
            skip_duplicates: true,
            column_choices: Vec::new(),
            account,
            category,
            default_type,
            debit_alias,
            credit_alias,
            currency,
            accounts,
            categories,
            syncing: false,
            focus: ModalFocus::new(cx),
            _subs: subs,
        }
    }

    #[cfg(test)]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Ask for a file with the system dialog, then load it.
    pub fn choose_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: None });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await {
                if let Some(path) = paths.into_iter().next() {
                    this.update_in(cx, |this, window, cx| this.load_file(&path.to_string_lossy(), window, cx)).ok();
                }
            }
        })
        .detach();
    }

    pub fn load_file(&mut self, path: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.error = None;
        // A first pass with no mapping just reads the headers.
        let headers = match parse_csv_preview(path, &CsvMapping::default()) {
            Ok(p) => p.headers,
            Err(e) => {
                self.error = Some(e);
                return cx.notify();
            }
        };
        self.path = Some(path.to_string());
        self.mapping = self.saved.get(&headers).cloned().unwrap_or_else(|| CsvMapping::from_headers(&headers));
        self.column_choices = headers
            .iter()
            .map(|h| {
                let current = self.mapping.columns.get(h).map(|f| field_value(*f).to_string()).unwrap_or_default();
                let state = choice(field_options(), Some(&current), window, cx);
                let header = h.clone();
                self._subs.push(cx.observe(&state, move |this, _, cx| this.column_changed(&header, cx)));
                (h.clone(), state)
            })
            .collect();
        self.apply_mapping_to_controls(window, cx);
        self.refresh_preview();
        cx.notify();
    }

    fn column_changed(&mut self, header: &str, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        let Some((_, state)) = self.column_choices.iter().find(|(h, _)| h == header) else {
            return;
        };
        let field = choice_value(state, cx).and_then(|v| field_from_value(&v));
        set_column(&mut self.mapping, header, field);
        self.syncing = true;
        // Assigning a field releases it from any other column; reflect that.
        for (h, state) in &self.column_choices {
            let want = self.mapping.columns.get(h).map(|f| field_value(*f)).unwrap_or("");
            if choice_value(state, cx).as_deref().unwrap_or("") != want {
                let index = field_options().iter().position(|(v, _)| v == want);
                state.update(cx, |s, cx| s.set_selected(index, cx));
            }
        }
        self.syncing = false;
        cx.notify();
    }

    /// Copy the mapping's defaults into the controls (after loading a saved mapping).
    fn apply_mapping_to_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.syncing = true;
        let m = &self.mapping;
        let index_of = |items: Vec<String>, wanted: &str| items.iter().position(|v| v == wanted);
        let accounts: Vec<String> = std::iter::once(String::new()).chain(self.accounts.iter().map(|a| a.id.clone())).collect();
        let categories: Vec<String> = std::iter::once(String::new()).chain(self.categories.iter().map(|c| c.id.clone())).collect();
        let pick = |state: &Entity<ChoiceState>, idx: Option<usize>, cx: &mut Context<Self>| state.update(cx, |s, cx| s.set_selected(idx, cx));
        pick(&self.account, index_of(accounts, m.default_account_id.as_deref().unwrap_or("")), cx);
        pick(&self.category, index_of(categories, m.default_category_id.as_deref().unwrap_or("")), cx);
        pick(
            &self.default_type,
            index_of(vec![String::new(), "expense".into(), "income".into()], m.default_transaction_type.as_deref().unwrap_or("")),
            cx,
        );
        let alias = |key: &str, default: &str| m.type_aliases.get(key).cloned().unwrap_or_else(|| default.to_string());
        pick(&self.debit_alias, index_of(vec!["expense".into(), "income".into()], &alias("debit", "expense")), cx);
        pick(&self.credit_alias, index_of(vec!["expense".into(), "income".into()], &alias("credit", "income")), cx);
        let currency = m.default_currency.clone().unwrap_or_else(|| "USD".into());
        self.currency.update(cx, |s, cx| s.set_value(currency, window, cx));
        self.syncing = false;
    }

    /// Controls -> mapping defaults.
    fn read_controls(&mut self, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        let blank = |v: Option<String>| v.filter(|s| !s.is_empty());
        self.mapping.default_account_id = blank(choice_value(&self.account, cx));
        self.mapping.default_category_id = blank(choice_value(&self.category, cx));
        self.mapping.default_transaction_type = blank(choice_value(&self.default_type, cx));
        let currency = text_of(&self.currency, cx).to_uppercase();
        self.mapping.default_currency = Some(if currency.is_empty() { "USD".into() } else { currency });
        for (key, state) in [("debit", &self.debit_alias), ("credit", &self.credit_alias)] {
            if let Some(to) = choice_value(state, cx) {
                self.mapping.type_aliases.insert(key.to_string(), to);
            }
        }
        cx.notify();
    }

    fn refresh_preview(&mut self) {
        let Some(path) = self.path.clone() else { return };
        match parse_csv_preview(&path, &self.mapping) {
            Ok(preview) => {
                self.duplicates = flag_duplicates(&preview.rows, &self.mapping);
                self.preview = Some(preview);
            }
            Err(e) => self.error = Some(e),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.path.is_some() && is_valid_for_preview(&self.mapping)
    }

    pub fn show_preview(&mut self, cx: &mut Context<Self>) {
        if !self.is_ready() {
            return;
        }
        self.error = None;
        self.refresh_preview();
        if self.error.is_none() {
            if let Some(p) = &self.preview {
                self.saved.insert(&p.headers, self.mapping.clone());
                // Remembering the mapping is a convenience; failing to write it must not block the import.
                let _ = self.saved.save_to(&self.saved_path);
            }
            self.stage = Stage::Preview;
        }
        cx.notify();
    }

    pub fn import(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.path.clone() else { return };
        let options = ImportOptions {
            default_account_id: self.mapping.default_account_id.clone(),
            default_category_id: self.mapping.default_category_id.clone(),
            default_currency: self.mapping.default_currency.clone(),
            skip_duplicates: self.skip_duplicates,
        };
        match commands::import_transactions(&self.db, path, self.mapping.clone(), options) {
            Ok(result) => {
                self.result = Some(result);
                self.error = None;
                self.stage = Stage::Result;
            }
            Err(e) => self.error = Some(e),
        }
        cx.notify();
    }

    pub fn ready_count(&self) -> usize {
        self.preview.as_ref().map_or(0, |p| valid_count(&p.rows, &self.duplicates))
    }

    fn mapping_stage(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::Div {
        let t = cx.omarchy().clone();
        let has_type = self.mapping.field_column(CsvField::Type).is_some();
        let separate = self.mapping.field_column(CsvField::IncomeAmount).is_some() || self.mapping.field_column(CsvField::ExpenseAmount).is_some();

        let file_row = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(rems(0.75))
            .child(match &self.path {
                Some(p) => div().text_color(t.secondary).overflow_hidden().child(p.clone()).into_any_element(),
                None => div().text_color(t.secondary).child("No file chosen").into_any_element(),
            })
            .child(
                button("csv-choose", if self.path.is_some() { "Change" } else { "Choose CSV File" }, ButtonVariant::Outline, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.choose_file(window, cx))),
            );

        let mut body = div().flex().flex_col().gap(rems(0.75)).child(file_row);
        if !self.column_choices.is_empty() {
            let mut table = div().id("csv-columns").flex().flex_col().max_h(rems(18.)).overflow_y_scroll().border_1().border_color(t.border).child(
                div().flex().bg(t.surface).font_weight(gpui_kit::FontWeight::BOLD).child(cell(None, "CSV Column")).child(cell(None, "Chelete Field")),
            );
            let choices = self.column_choices.clone();
            for (i, (header, state)) in choices.iter().enumerate() {
                table = table.child(
                    div()
                        .flex()
                        .items_center()
                        .py(rems(0.125))
                        .border_t_1()
                        .border_color(t.border)
                        .child(cell(None, header.clone()))
                        .child(cell(None, select(("csv-col", i), state, window, cx))),
                );
            }
            body = body.child(table);

            let mut defaults = div()
                .flex()
                .flex_wrap()
                .gap(rems(0.5))
                .child(field("Default Account", select("csv-account", &self.account, window, cx), cx))
                .child(field("Default Category", select("csv-category", &self.category, window, cx), cx))
                .child(field("Default Currency", input("csv-currency", &self.currency, window, cx), cx))
                .child(field("Default Type", select("csv-type", &self.default_type, window, cx), cx));
            if has_type {
                defaults = defaults
                    .child(field("Debit means", select("csv-debit", &self.debit_alias, window, cx), cx))
                    .child(field("Credit means", select("csv-credit", &self.credit_alias, window, cx), cx));
            }
            body = body.child(defaults).child(dim(
                if separate {
                    "Map at least Date and Description plus an amount. Separate Income and Expense columns also satisfy the type requirement."
                } else {
                    "Map at least Date, Description, and an Amount + Type field to continue."
                },
                cx,
            ));
        }
        body.child(
            div()
                .flex()
                .justify_end()
                .gap(rems(0.5))
                .child(button("csv-cancel", "Cancel", ButtonVariant::Outline, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(CsvImportEvent::Cancel))))
                .child(
                    button("csv-preview", "Preview", ButtonVariant::Primary, cx)
                        .disabled(!self.is_ready())
                        .on_click(cx.listener(|this, _, _, cx| this.show_preview(cx))),
                ),
        )
    }

    fn preview_stage(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let t = cx.omarchy().clone();
        let Some(preview) = &self.preview else {
            return div();
        };
        let mut table = div().id("csv-preview-rows").flex().flex_col().max_h(rems(20.)).overflow_y_scroll().border_1().border_color(t.border).child(
            div()
                .flex()
                .bg(t.surface)
                .font_weight(gpui_kit::FontWeight::BOLD)
                .child(cell(Some(6.), "Date"))
                .child(cell(None, "Description"))
                .child(cell(Some(7.), "Amount"))
                .child(cell(Some(5.), "Type"))
                .child(cell(Some(8.), "Account"))
                .child(cell(Some(8.), "Category"))
                .child(cell(Some(10.), "Status")),
        );
        for (row, dup) in preview.rows.iter().zip(&self.duplicates) {
            let status = if !row.errors.is_empty() {
                div().text_color(t.danger).child(row.errors.join("; "))
            } else if *dup {
                div().text_color(t.warning).child("Duplicate")
            } else {
                div().text_color(t.success).child("OK")
            };
            table = table.child(
                div()
                    .flex()
                    .py(rems(0.125))
                    .border_t_1()
                    .border_color(t.border)
                    .child(cell(Some(6.), row.date.clone().unwrap_or_else(|| "—".into())))
                    .child(cell(None, row.description.clone().unwrap_or_else(|| "—".into())))
                    .child(cell(Some(7.), row.amount_cents.map(format_money).unwrap_or_else(|| "—".into())))
                    .child(cell(Some(5.), type_label(row)))
                    .child(cell(Some(8.), row.account.clone().unwrap_or_else(|| if self.mapping.default_account_id.is_some() { "(default)".into() } else { "—".into() })))
                    .child(cell(Some(8.), row.category.clone().unwrap_or_else(|| if self.mapping.default_category_id.is_some() { "(default)".into() } else { "—".into() })))
                    .child(cell(Some(10.), status)),
            );
        }
        let ready = self.ready_count();
        let screen = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(dim(format!("{} preview rows (of {} total)", preview.rows.len(), preview.total_rows), cx))
            .child(table)
            .child(check(
                "csv-skip-dupes",
                "Skip exact duplicates (same date, amount, description, account)",
                if self.skip_duplicates { CheckboxState::Checked } else { CheckboxState::Unchecked },
                move |_, cx| screen.update(cx, |s, cx| { s.skip_duplicates = !s.skip_duplicates; cx.notify(); }),
                cx,
            ))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(button("csv-cancel", "Cancel", ButtonVariant::Outline, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(CsvImportEvent::Cancel))))
                    .child(button("csv-back", "Back", ButtonVariant::Outline, cx).on_click(cx.listener(|this, _, _, cx| { this.stage = Stage::Mapping; cx.notify(); })))
                    .child(
                        button("csv-import", format!("Import {ready} Transactions"), ButtonVariant::Primary, cx)
                            .disabled(ready == 0)
                            .on_click(cx.listener(|this, _, _, cx| this.import(cx))),
                    ),
            )
    }

    fn result_stage(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let t = cx.omarchy().clone();
        let Some(result) = &self.result else {
            return div();
        };
        let stat = |value: usize, label: &str, color| {
            div().flex_1().flex().flex_col().items_center().child(div().text_size(rems(1.5)).text_color(color).child(value.to_string())).child(dim(label.to_string(), cx))
        };
        let mut body = div().flex().flex_col().gap(rems(0.75)).child(
            div()
                .flex()
                .child(stat(result.imported, "Imported", t.success))
                .child(stat(result.skipped, "Skipped", t.secondary))
                .child(stat(result.errors.len(), "Errors", if result.errors.is_empty() { t.secondary } else { t.danger })),
        );
        if !result.errors.is_empty() {
            body = body.child(
                div()
                    .id("csv-errors")
                    .max_h(rems(10.))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(rems(0.25))
                    .children(result.errors.iter().map(|e| div().text_color(t.danger).child(e.clone()))),
            );
        }
        body.child(
            div().flex().justify_end().gap(rems(0.5)).child(button("csv-done", "Done", ButtonVariant::Primary, cx).on_click(cx.listener(|_, _, _, cx| cx.emit(CsvImportEvent::Done)))),
        )
    }
}

impl Render for CsvImportDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.focus.ensure(window, cx);
        let error = self.error.clone().map(|e| alert(e, Status::Error, cx));
        let stage = match self.stage {
            Stage::Mapping => self.mapping_stage(window, cx),
            Stage::Preview => self.preview_stage(cx),
            Stage::Result => self.result_stage(cx),
        };
        let body = div().flex().flex_col().gap(rems(0.75)).children(error).child(stage);
        modal_sized(
            52.,
            "Import Transactions from CSV",
            body,
            cx.listener(|_, _, _, cx| cx.emit(CsvImportEvent::Cancel)),
            cx,
        ).track_focus(&self.focus.handle)
            .on_action(cx.listener(|_, _: &super::ModalCancel, _, cx| cx.emit(CsvImportEvent::Cancel)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use gpui_kit::{TestAppContext, VisualTestContext};

    struct Env {
        dir: PathBuf,
    }

    impl Env {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("chelete-csv-ui-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            Self { dir }
        }
        fn csv(&self, name: &str, body: &str) -> String {
            let path = self.dir.join(name);
            std::fs::write(&path, body).unwrap();
            path.to_string_lossy().to_string()
        }
        fn mappings(&self) -> PathBuf {
            self.dir.join("mappings.json")
        }
    }

    impl Drop for Env {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.dir).ok();
        }
    }

    fn setup<'a>(cx: &'a mut TestAppContext, env: &Env) -> (Entity<CsvImportDialog>, Arc<DbState>, &'a mut VisualTestContext) {
        init(cx);
        let db = seeded_db();
        let handle = db.clone();
        let accounts = commands::get_accounts(&db).unwrap();
        let categories = commands::get_categories(&db).unwrap();
        let path = env.mappings();
        let (dialog, vcx) = cx.add_window_view(move |window, cx| CsvImportDialog::new(db, accounts, categories, path, window, cx));
        (dialog, handle, vcx)
    }

    fn load(dialog: &Entity<CsvImportDialog>, path: &str, cx: &mut VisualTestContext) {
        let path = path.to_string();
        cx.update(|window, cx| dialog.update(cx, |d, cx| d.load_file(&path, window, cx)));
    }

    const BANK: &str = "Date,Payee,Debit,Credit,Account\n2026-03-01,Coffee,4.50,,Main\n2026-03-01,Coffee,4.50,,Main\n2026-03-02,Salary,,2000.00,Main\n";

    #[gpui_kit::test]
    fn recognises_a_bank_csv_and_imports_it(cx: &mut TestAppContext) {
        let env = Env::new("bank");
        let (dialog, db, cx) = setup(cx, &env);
        let account = commands::get_accounts(&db).unwrap().remove(0);
        let before = commands::get_transactions(&db).unwrap().len();
        load(&dialog, &env.csv("bank.csv", BANK), cx);

        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert!(d.is_ready(), "Date/Payee/Debit/Credit are recognised without any clicks");
            assert_eq!(d.column_choices.len(), 5);
            assert_eq!(d.stage(), Stage::Mapping);
        });
        // The CSV names an account that does not exist, so fall back to a default.
        let acct = cx.update(|_, cx| dialog.read(cx).account.clone());
        cx.update(|_, cx| acct.update(cx, |c, cx| c.set_selected(Some(1), cx)));
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.show_preview(cx)));
        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert_eq!(d.stage(), Stage::Preview);
            assert_eq!(d.duplicates, vec![false, true, false]);
            assert_eq!(d.ready_count(), 2, "the repeated coffee is held back");
        });
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.import(cx)));
        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert_eq!(d.stage(), Stage::Result);
            let result = d.result.as_ref().unwrap();
            assert_eq!((result.imported, result.skipped), (2, 1));
            assert!(result.errors.is_empty(), "{:?}", result.errors);
        });
        let after = commands::get_transactions(&db).unwrap();
        assert_eq!(after.len(), before + 2);
        let salary = after.iter().find(|t| t.description == "Salary").expect("imported");
        assert_eq!((salary.amount, salary.transaction_type.as_str()), (200000, "income"));
        assert_eq!(salary.account_id, account.id);
    }

    #[gpui_kit::test]
    fn without_skipping_duplicates_both_coffees_import(cx: &mut TestAppContext) {
        let env = Env::new("nodedupe");
        let (dialog, db, cx) = setup(cx, &env);
        let before = commands::get_transactions(&db).unwrap().len();
        load(&dialog, &env.csv("bank.csv", BANK), cx);
        let acct = cx.update(|_, cx| dialog.read(cx).account.clone());
        cx.update(|_, cx| acct.update(cx, |c, cx| c.set_selected(Some(1), cx)));
        cx.update(|_, cx| dialog.update(cx, |d, cx| { d.skip_duplicates = false; d.show_preview(cx); d.import(cx) }));
        assert_eq!(commands::get_transactions(&db).unwrap().len(), before + 3);
    }

    #[gpui_kit::test]
    fn assigning_a_field_releases_it_from_the_other_column(cx: &mut TestAppContext) {
        let env = Env::new("conflict");
        let (dialog, _db, cx) = setup(cx, &env);
        load(&dialog, &env.csv("bank.csv", BANK), cx);
        let payee = cx.update(|_, cx| dialog.read(cx).column_choices.iter().find(|(h, _)| h == "Payee").unwrap().1.clone());
        let date_index = field_options().iter().position(|(v, _)| v == "date").unwrap();
        cx.update(|_, cx| payee.update(cx, |c, cx| c.set_selected(Some(date_index), cx)));
        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert_eq!(d.mapping.field_column(CsvField::Date), Some("Payee"));
            assert_eq!(d.mapping.columns.get("Date"), None, "old date column released");
            let released = d.column_choices.iter().find(|(h, _)| h == "Date").unwrap();
            assert_eq!(choice_value(&released.1, cx).as_deref(), Some(""), "its dropdown shows unmapped");
            assert!(!d.is_ready(), "description is now unmapped");
        });
    }

    #[gpui_kit::test]
    fn cannot_preview_until_the_mapping_is_complete(cx: &mut TestAppContext) {
        let env = Env::new("incomplete");
        let (dialog, _db, cx) = setup(cx, &env);
        load(&dialog, &env.csv("odd.csv", "foo,bar\n1,2\n"), cx);
        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert!(!d.is_ready());
        });
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.show_preview(cx)));
        cx.update(|_, cx| assert_eq!(dialog.read(cx).stage(), Stage::Mapping));
    }

    #[gpui_kit::test]
    fn a_missing_file_shows_an_error_and_keeps_going(cx: &mut TestAppContext) {
        let env = Env::new("missing");
        let (dialog, _db, cx) = setup(cx, &env);
        load(&dialog, "/nonexistent/file.csv", cx);
        cx.update(|_, cx| {
            let d = dialog.read(cx);
            assert!(d.error.is_some());
            assert!(d.path.is_none());
            assert!(!d.is_ready());
        });
    }

    #[gpui_kit::test]
    fn rows_with_bad_data_are_reported_not_imported(cx: &mut TestAppContext) {
        let env = Env::new("baddata");
        let (dialog, db, cx) = setup(cx, &env);
        let before = commands::get_transactions(&db).unwrap().len();
        let csv = env.csv("bad.csv", "Date,Description,Amount,Type\n2026-03-01,Fine,10.00,expense\nnot-a-date,Broken,5.00,expense\n");
        load(&dialog, &csv, cx);
        let acct = cx.update(|_, cx| dialog.read(cx).account.clone());
        cx.update(|_, cx| acct.update(cx, |c, cx| c.set_selected(Some(1), cx)));
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.show_preview(cx)));
        cx.update(|_, cx| assert_eq!(dialog.read(cx).ready_count(), 1));
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.import(cx)));
        assert_eq!(commands::get_transactions(&db).unwrap().len(), before + 1);
    }

    #[gpui_kit::test]
    fn the_mapping_is_remembered_for_the_same_layout(cx: &mut TestAppContext) {
        let env = Env::new("remember");
        let (dialog, db, cx) = setup(cx, &env);
        load(&dialog, &env.csv("bank.csv", BANK), cx);
        let acct = cx.update(|_, cx| dialog.read(cx).account.clone());
        cx.update(|_, cx| acct.update(cx, |c, cx| c.set_selected(Some(2), cx)));
        let chosen = cx.update(|_, cx| dialog.read(cx).mapping.default_account_id.clone());
        assert!(chosen.is_some());
        cx.update(|_, cx| dialog.update(cx, |d, cx| d.show_preview(cx)));
        assert!(env.mappings().exists(), "mapping written on preview");

        // A second dialog on a same-shaped file starts from the saved choices.
        let (db2, accounts, categories) = (db.clone(), commands::get_accounts(&db).unwrap(), commands::get_categories(&db).unwrap());
        let path = env.mappings();
        let second = cx.update(|window, cx| cx.new(|cx| CsvImportDialog::new(db2, accounts, categories, path, window, cx)));
        load(&second, &env.csv("bank-again.csv", BANK), cx);
        cx.update(|_, cx| {
            let d = second.read(cx);
            assert!(d.is_ready());
            assert!(d.mapping.default_account_id.is_some(), "default account restored from the saved mapping");
            assert_eq!(choice_value(&d.account, cx).as_deref(), d.mapping.default_account_id.as_deref());
        });
    }
}
