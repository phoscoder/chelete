use super::widgets::*;
use chelete_lib::commands::{self, Account, CreateAccountRequest, UpdateAccountRequest};
use chelete_lib::database::DbState;
use chelete_lib::format::{account_type_label, format_balance, parse_cents};
use gpui_kit::base::input::InputState;
use gpui_kit::{
    div, prelude::*, rems, Context, Entity, EventEmitter, IntoElement, Render, Subscription,
    SharedString, Window,
};
use gpui_omarchy::{
    button, empty_state, icon, input, panel, select, ActiveTheme as _, ButtonVariant,
    ChoiceState, IconName,
};
use std::sync::Arc;

enum Dialog {
    Form(Entity<AccountForm>),
    Transfer(Entity<TransferForm>),
    Confirm(Entity<ConfirmDialog>, String),
}

pub struct AccountsScreen {
    db: Arc<DbState>,
    accounts: Result<Vec<Account>, String>,
    dialog: Option<Dialog>,
    _dialog_sub: Option<Subscription>,
}

impl EventEmitter<ScreenEvent> for AccountsScreen {}

impl AccountsScreen {
    pub fn new(db: Arc<DbState>) -> Self {
        let accounts = commands::get_accounts(&db);
        Self { db, accounts, dialog: None, _dialog_sub: None }
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.accounts = commands::get_accounts(&self.db);
        cx.notify();
    }

    pub fn accounts(&self) -> &[Account] {
        self.accounts.as_deref().unwrap_or(&[])
    }

    #[cfg(test)]
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn open_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_form(None, window, cx);
    }

    fn open_form(&mut self, editing: Option<Account>, window: &mut Window, cx: &mut Context<Self>) {
        let form = cx.new(|cx| AccountForm::new(self.db.clone(), editing, window, cx));
        self._dialog_sub = Some(cx.subscribe(&form, Self::on_form_event));
        self.dialog = Some(Dialog::Form(form));
        cx.notify();
    }

    fn open_transfer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let accounts = self.accounts().to_vec();
        let form = cx.new(|cx| TransferForm::new(self.db.clone(), accounts, window, cx));
        self._dialog_sub = Some(cx.subscribe(&form, Self::on_form_event));
        self.dialog = Some(Dialog::Transfer(form));
        cx.notify();
    }

    fn on_form_event<T: 'static>(&mut self, _: Entity<T>, event: &FormEvent, cx: &mut Context<Self>) {
        match event {
            FormEvent::Cancel => {}
            FormEvent::Saved(message) => {
                self.accounts = commands::get_accounts(&self.db);
                cx.emit(ScreenEvent::ok(message.clone()));
            }
            FormEvent::Failed(message) => {
                cx.emit(ScreenEvent::error(message.clone()));
                return;
            }
        }
        self.dialog = None;
        self._dialog_sub = None;
        cx.notify();
    }

    fn ask_delete(&mut self, id: String, cx: &mut Context<Self>) {
        let Some(name) = self.accounts().iter().find(|a| a.id == id).map(|a| a.name.clone()) else {
            return;
        };
        let records = commands::count_account_transactions(&self.db, &id).unwrap_or(0);
        let message = if records == 0 {
            format!("This will permanently remove {name}. This action cannot be undone.")
        } else {
            format!(
                "This will permanently remove {name} and its {}. This action cannot be undone.",
                plural(records, "transaction")
            )
        };
        let dialog = cx.new(|cx| ConfirmDialog::new("Delete account?", message, "Delete", cx));
        self._dialog_sub = Some(cx.subscribe(&dialog, |this, _, event: &ConfirmEvent, cx| {
            if *event == ConfirmEvent::Confirm {
                if let Some(Dialog::Confirm(_, id)) = this.dialog.take() {
                    this.delete(id, cx);
                }
            }
            this.dialog = None;
            this._dialog_sub = None;
            cx.notify();
        }));
        self.dialog = Some(Dialog::Confirm(dialog, id));
        cx.notify();
    }

    fn delete(&mut self, id: String, cx: &mut Context<Self>) {
        let name = self.accounts().iter().find(|a| a.id == id).map(|a| a.name.clone()).unwrap_or_default();
        match commands::delete_account(&self.db, id) {
            Ok(()) => {
                self.accounts = commands::get_accounts(&self.db);
                cx.emit(ScreenEvent::ok(format!("Deleted account {name}")));
            }
            Err(e) => cx.emit(ScreenEvent::error(format!("Could not delete account: {e}"))),
        }
        cx.notify();
    }
}

impl Render for AccountsScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let actions = div()
            .flex()
            .gap(rems(0.5))
            .child(
                button("transfer", "Transfer", ButtonVariant::Outline, cx)
                    .child(icon(IconName::ArrowLeftRight).size(rems(0.875)))
                    .on_click(cx.listener(|this, _, window, cx| this.open_transfer(window, cx))),
            )
            .child(
                button("add-account", "+ Add", ButtonVariant::Primary, cx)
                    .on_click(cx.listener(|this, _, window, cx| this.open_add(window, cx))),
            );

        let body = match &self.accounts {
            Err(e) => div().text_color(t.danger).child(format!("Could not load accounts: {e}")).into_any_element(),
            Ok(list) if list.is_empty() => {
                empty_state("No accounts yet", "Add one to get started.", cx).into_any_element()
            }
            Ok(list) => {
                let cards: Vec<_> = list
                    .iter()
                    .map(|a| {
                        let id = a.id.clone();
                        let editing = a.clone();
                        panel(account_type_label(&a.account_type), cx)
                            .w(rems(16.))
                            .child(div().text_size(rems(0.875)).child(a.name.clone()))
                            .child(
                                div()
                                    .text_size(rems(1.25))
                                    .text_color(if a.balance < 0 { t.danger } else { t.foreground })
                                    .child(format_balance(a.balance)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(rems(0.5))
                                    .child(
                                        button(SharedString::from(format!("edit-account-{}", a.id)), "Edit", ButtonVariant::Outline, cx)
                                            .on_click(cx.listener(move |this, _, window, cx| this.open_form(Some(editing.clone()), window, cx))),
                                    )
                                    .child(
                                        button(SharedString::from(format!("delete-account-{}", a.id)), "Delete", ButtonVariant::Danger, cx)
                                            .on_click(cx.listener(move |this, _, _, cx| this.ask_delete(id.clone(), cx))),
                                    ),
                            )
                    })
                    .collect();
                div().flex().flex_wrap().gap(rems(1.)).children(cards).into_any_element()
            }
        };

        let dialog = self.dialog.as_ref().map(|d| match d {
            Dialog::Form(form) => form.clone().into_any_element(),
            Dialog::Transfer(form) => form.clone().into_any_element(),
            Dialog::Confirm(dialog, _) => dialog.clone().into_any_element(),
        });

        div()
            .id("accounts")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .p(rems(1.5))
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(page_header("Accounts", actions, cx))
            .child(body)
            .children(dialog)
    }
}

#[derive(Clone, Debug)]
pub enum FormEvent {
    Cancel,
    Saved(String),
    Failed(String),
}

/// Adds a new account, or edits the name, type and balance of `editing`.
pub struct AccountForm {
    db: Arc<DbState>,
    editing: Option<Account>,
    name: Entity<InputState>,
    kind: Entity<ChoiceState>,
    balance: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<FormEvent> for AccountForm {}

impl AccountForm {
    fn new(db: Arc<DbState>, editing: Option<Account>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name_value = editing.as_ref().map(|a| a.name.clone()).unwrap_or_default();
        let balance_value = editing.as_ref().map(|a| format!("{:.2}", a.balance as f64 / 100.0)).unwrap_or_default();
        let kind = editing.as_ref().map(|a| a.account_type.clone()).unwrap_or_else(|| "cash".into());
        let name = text_input_with("e.g. Main Bank", &name_value, window, cx);
        let balance = text_input_with("0.00", &balance_value, window, cx);
        // When editing, the balance is usually what needs correcting.
        if editing.is_some() { &balance } else { &name }.update(cx, |s, cx| s.focus(window, cx));
        let _subs = submit_on_enter(&[&name, &balance], Self::submit, cx);
        Self {
            db,
            editing,
            name,
            kind: choice(account_type_options(), Some(&kind), window, cx),
            balance,
            _subs,
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let name = text_of(&self.name, cx);
        if name.is_empty() {
            return;
        }
        let balance_text = text_of(&self.balance, cx);
        let balance = if balance_text.is_empty() {
            0
        } else {
            match parse_cents(&balance_text) {
                Some(c) => c,
                None => return cx.emit(FormEvent::Failed("Balance is not a valid amount".into())),
            }
        };
        let account_type = choice_value(&self.kind, cx).unwrap_or_else(|| "cash".into());
        if let Some(existing) = &self.editing {
            let request = UpdateAccountRequest {
                id: existing.id.clone(),
                name: Some(name),
                account_type: Some(account_type),
                currency: None,
                balance: Some(balance),
                color: None,
                icon: None,
                is_active: None,
            };
            return match commands::update_account(&self.db, request) {
                Ok(a) => cx.emit(FormEvent::Saved(format!("Saved account {}", a.name))),
                Err(e) => cx.emit(FormEvent::Failed(format!("Could not save account: {e}"))),
            };
        }
        let request = CreateAccountRequest {
            name,
            account_type,
            currency: "USD".into(),
            balance,
            color: None,
            icon: None,
        };
        match commands::create_account(&self.db, request) {
            Ok(a) => cx.emit(FormEvent::Saved(format!("Added account {}", a.name))),
            Err(e) => cx.emit(FormEvent::Failed(format!("Could not add account: {e}"))),
        }
    }
}

impl Render for AccountForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.editing.is_some();
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(field("Name", input("account-name", &self.name, window, cx), cx))
            .child(field("Type", select("account-type", &self.kind, window, cx), cx))
            .child(field(if editing { "Balance" } else { "Starting Balance" }, input("account-balance", &self.balance, window, cx), cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(
                        button("add-cancel", "Cancel", ButtonVariant::Outline, cx)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel))),
                    )
                    .child(
                        button("add-submit", if editing { "Save" } else { "Add" }, ButtonVariant::Primary, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                    ),
            );
        modal(if editing { "Edit Account" } else { "Add Account" }, body, cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel)), cx).on_action(cx.listener(|_, _: &super::ModalCancel, _, cx| cx.emit(FormEvent::Cancel)))
    }
}

pub struct TransferForm {
    db: Arc<DbState>,
    accounts: Vec<Account>,
    from: Entity<ChoiceState>,
    to: Entity<ChoiceState>,
    amount: Entity<InputState>,
    notes: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<FormEvent> for TransferForm {}

impl TransferForm {
    fn new(db: Arc<DbState>, accounts: Vec<Account>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let items: Vec<(String, String)> =
            accounts.iter().map(|a| (a.id.clone(), a.name.clone())).collect();
        let first = accounts.first().map(|a| a.id.clone());
        let second = accounts.get(1).or(accounts.first()).map(|a| a.id.clone());
        let amount = text_input("0.00", window, cx);
        amount.update(cx, |s, cx| s.focus(window, cx));
        let notes = text_input("e.g. Monthly savings", window, cx);
        let _subs = submit_on_enter(&[&amount, &notes], Self::submit, cx);
        Self {
            db,
            from: choice(items.clone(), first.as_deref(), window, cx),
            to: choice(items, second.as_deref(), window, cx),
            amount,
            notes,
            accounts,
            _subs,
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let (Some(from), Some(to)) = (choice_value(&self.from, cx), choice_value(&self.to, cx)) else {
            return;
        };
        if from == to {
            return cx.emit(FormEvent::Failed("Pick two different accounts".into()));
        }
        let amount_text = text_of(&self.amount, cx);
        let cents = match parse_cents(&amount_text) {
            Some(c) if c > 0 => c,
            _ => return cx.emit(FormEvent::Failed("Enter an amount greater than zero".into())),
        };
        let notes = text_of(&self.notes, cx);
        let to_name = self.accounts.iter().find(|a| a.id == to).map(|a| a.name.clone()).unwrap_or_default();
        match commands::transfer(
            &self.db,
            from,
            to,
            cents,
            "USD".into(),
            (!notes.is_empty()).then_some(notes),
        ) {
            Ok(_) => cx.emit(FormEvent::Saved(format!("Transferred ${amount_text} to {to_name}"))),
            Err(e) => cx.emit(FormEvent::Failed(format!("Transfer failed: {e}"))),
        }
    }
}

impl Render for TransferForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(
                div()
                    .flex()
                    .gap(rems(0.5))
                    .child(field("From", select("transfer-from", &self.from, window, cx), cx))
                    .child(field("To", select("transfer-to", &self.to, window, cx), cx)),
            )
            .child(field("Amount", input("transfer-amount", &self.amount, window, cx), cx))
            .child(field("Notes", input("transfer-notes", &self.notes, window, cx), cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(
                        button("transfer-cancel", "Cancel", ButtonVariant::Outline, cx)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel))),
                    )
                    .child(
                        button("transfer-submit", "Transfer", ButtonVariant::Primary, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                    ),
            );
        modal("Transfer Money", body, cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel)), cx).on_action(cx.listener(|_, _: &super::ModalCancel, _, cx| cx.emit(FormEvent::Cancel)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use gpui_kit::{TestAppContext, VisualTestContext};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn setup(cx: &mut TestAppContext) -> (Entity<AccountsScreen>, &mut VisualTestContext, Rc<RefCell<Vec<ScreenEvent>>>) {
        init(cx);
        let db = seeded_db();
        let (screen, vcx) = cx.add_window_view(|_, _| AccountsScreen::new(db));
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        vcx.update(|_, cx| {
            cx.subscribe(&screen, move |_, e: &ScreenEvent, _| sink.borrow_mut().push(e.clone())).detach();
        });
        (screen, vcx, events)
    }

    fn open_add(screen: &Entity<AccountsScreen>, cx: &mut VisualTestContext) -> Entity<AccountForm> {
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Form(f)) => f.clone(),
            _ => panic!("add form should be open"),
        })
    }

    fn type_into(input: &Entity<InputState>, text: &str, cx: &mut VisualTestContext) {
        cx.update(|window, cx| input.update(cx, |i, cx| i.set_value(text, window, cx)));
    }

    #[gpui_kit::test]
    fn lists_seeded_accounts(cx: &mut TestAppContext) {
        let (screen, cx, _) = setup(cx);
        cx.update(|_, cx| assert_eq!(screen.read(cx).accounts().len(), 4));
    }

    #[gpui_kit::test]
    fn add_account_saves_and_closes(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        let form = open_add(&screen, cx);
        let (name, balance) = cx.update(|_, cx| (form.read(cx).name.clone(), form.read(cx).balance.clone()));
        type_into(&name, "Rainy Day", cx);
        type_into(&balance, "1,250.50", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let screen = screen.read(cx);
            assert!(!screen.has_dialog(), "dialog closes after saving");
            let added = screen.accounts().iter().find(|a| a.name == "Rainy Day").expect("account saved");
            assert_eq!(added.balance, 125050);
            assert_eq!(added.account_type, "cash");
        });
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: true, .. })));
    }

    #[gpui_kit::test]
    fn blank_name_is_ignored_and_bad_balance_reports(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        let form = open_add(&screen, cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| assert!(screen.read(cx).has_dialog(), "blank name keeps the form open"));

        let (name, balance) = cx.update(|_, cx| (form.read(cx).name.clone(), form.read(cx).balance.clone()));
        type_into(&name, "Oops", cx);
        type_into(&balance, "twelve", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            assert!(screen.read(cx).has_dialog(), "invalid balance keeps the form open");
            assert_eq!(screen.read(cx).accounts().len(), 4);
        });
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: false, .. })));
    }

    #[gpui_kit::test]
    fn transfer_moves_money_between_accounts(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        let before: Vec<i64> = cx.update(|_, cx| screen.read(cx).accounts().iter().map(|a| a.balance).collect());
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_transfer(window, cx)));
        let form = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Transfer(f)) => f.clone(),
            _ => panic!("transfer form should be open"),
        });
        let amount = cx.update(|_, cx| form.read(cx).amount.clone());
        type_into(&amount, "10.00", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let screen = screen.read(cx);
            assert!(!screen.has_dialog());
            let after: Vec<i64> = screen.accounts().iter().map(|a| a.balance).collect();
            assert_eq!(after[0], before[0] - 1000, "source debited");
            assert_eq!(after[1], before[1] + 1000, "destination credited");
            assert_eq!(after.iter().sum::<i64>(), before.iter().sum::<i64>(), "total unchanged");
        });
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: true, .. })));
    }

    #[gpui_kit::test]
    fn transfer_rejects_zero_amount(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_transfer(window, cx)));
        let form = cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Transfer(f)) => f.clone(),
            _ => panic!("transfer form should be open"),
        });
        let amount = cx.update(|_, cx| form.read(cx).amount.clone());
        type_into(&amount, "0", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| assert!(screen.read(cx).has_dialog()));
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: false, .. })));
    }

    fn ask_delete(screen: &Entity<AccountsScreen>, id: &str, cx: &mut VisualTestContext) -> Entity<ConfirmDialog> {
        cx.update(|_, cx| screen.update(cx, |s, cx| s.ask_delete(id.to_string(), cx)));
        cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Confirm(d, _)) => d.clone(),
            _ => panic!("confirm dialog should be open"),
        })
    }

    #[gpui_kit::test]
    fn delete_asks_first_and_removes_on_confirm(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        let id = cx.update(|_, cx| screen.read(cx).accounts()[0].id.clone());
        let dialog = ask_delete(&screen, &id, cx);
        cx.update(|_, cx| assert_eq!(screen.read(cx).accounts().len(), 4, "nothing deleted until confirmed"));
        cx.update(|_, cx| dialog.update(cx, |_, cx| cx.emit(ConfirmEvent::Confirm)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_dialog());
            assert_eq!(s.accounts().len(), 3);
            assert!(s.accounts().iter().all(|a| a.id != id));
        });
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: true, .. })));
    }

    #[gpui_kit::test]
    fn cancelling_the_delete_keeps_the_account(cx: &mut TestAppContext) {
        let (screen, cx, _) = setup(cx);
        let id = cx.update(|_, cx| screen.read(cx).accounts()[0].id.clone());
        let dialog = ask_delete(&screen, &id, cx);
        cx.update(|_, cx| dialog.update(cx, |_, cx| cx.emit(ConfirmEvent::Cancel)));
        cx.update(|_, cx| {
            assert!(!screen.read(cx).has_dialog());
            assert_eq!(screen.read(cx).accounts().len(), 4);
        });
    }

    fn open_edit(screen: &Entity<AccountsScreen>, account: &Account, cx: &mut VisualTestContext) -> Entity<AccountForm> {
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_form(Some(account.clone()), window, cx)));
        cx.update(|_, cx| match screen.read(cx).dialog.as_ref() {
            Some(Dialog::Form(f)) => f.clone(),
            _ => panic!("edit form should be open"),
        })
    }

    #[gpui_kit::test]
    fn edit_corrects_the_balance_in_place(cx: &mut TestAppContext) {
        let (screen, cx, _) = setup(cx);
        let account = cx.update(|_, cx| screen.read(cx).accounts()[1].clone());
        let form = open_edit(&screen, &account, cx);
        let (name, balance) = cx.update(|_, cx| {
            let f = form.read(cx);
            assert_eq!(f.name.read(cx).value().as_ref(), account.name);
            assert_eq!(f.balance.read(cx).value().as_ref(), format!("{:.2}", account.balance as f64 / 100.0));
            (f.name.clone(), f.balance.clone())
        });
        type_into(&name, "Renamed", cx);
        type_into(&balance, "1419.27", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_dialog());
            assert_eq!(s.accounts().len(), 4, "edited, not added");
            let saved = s.accounts().iter().find(|a| a.id == account.id).unwrap();
            assert_eq!((saved.name.as_str(), saved.balance), ("Renamed", 141927));
            assert_eq!(saved.account_type, account.account_type, "type kept");
        });
    }

    #[gpui_kit::test]
    fn edit_with_an_invalid_balance_keeps_the_form_open(cx: &mut TestAppContext) {
        let (screen, cx, events) = setup(cx);
        let account = cx.update(|_, cx| screen.read(cx).accounts()[0].clone());
        let form = open_edit(&screen, &account, cx);
        let balance = cx.update(|_, cx| form.read(cx).balance.clone());
        type_into(&balance, "lots", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(s.has_dialog());
            assert_eq!(s.accounts().iter().find(|a| a.id == account.id).unwrap().balance, account.balance);
        });
        assert!(matches!(events.borrow().last(), Some(ScreenEvent::Toast { ok: false, .. })));
    }

    #[gpui_kit::test]
    fn escape_closes_the_open_form(cx: &mut TestAppContext) {
        let (screen, cx, _) = setup(cx);
        open_add(&screen, cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("escape");
        cx.update(|_, cx| assert!(!screen.read(cx).has_dialog(), "escape dismissed the form"));
        cx.update(|_, cx| assert_eq!(screen.read(cx).accounts().len(), 4, "nothing was saved"));
    }

    #[gpui_kit::test]
    fn enter_in_a_field_submits_the_form(cx: &mut TestAppContext) {
        let (screen, cx, _) = setup(cx);
        let form = open_add(&screen, cx);
        let name = cx.update(|_, cx| form.read(cx).name.clone());
        type_into(&name, "Keyboard Only", cx);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_dialog());
            assert!(s.accounts().iter().any(|a| a.name == "Keyboard Only"));
        });
    }
}
