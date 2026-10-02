use super::accounts::FormEvent;
use super::icons::{category_icon, CATEGORY_ICONS, DEFAULT_ICON};
use super::widgets::*;
use chelete_lib::commands::{self, Category, CreateCategoryRequest, Transaction, UpdateCategoryRequest};
use chelete_lib::database::DbState;
use gpui_kit::base::input::InputState;
use gpui_kit::{
    div, prelude::*, px, rems, Context, Entity, EventEmitter, Hsla, IntoElement, Render,
    SharedString, Subscription, Window,
};
use gpui_omarchy::{
    button, icon, icon_button, input, select, ActiveTheme as _, ButtonVariant, ChoiceState,
    IconName,
};
use std::collections::HashMap;
use std::sync::Arc;

/// How many transactions reference each category.
pub fn record_counts(transactions: &[Transaction]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for t in transactions {
        if let Some(id) = &t.category_id {
            *counts.entry(id.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn records_label(count: usize) -> String {
    format!("{count} {}", if count == 1 { "record" } else { "records" })
}

pub struct CategoriesScreen {
    db: Arc<DbState>,
    categories: Result<Vec<Category>, String>,
    counts: HashMap<String, usize>,
    form: Option<Entity<CategoryForm>>,
    _form_sub: Option<Subscription>,
}

impl EventEmitter<ScreenEvent> for CategoriesScreen {}

impl CategoriesScreen {
    pub fn new(db: Arc<DbState>) -> Self {
        let mut screen = Self {
            db,
            categories: Ok(Vec::new()),
            counts: HashMap::new(),
            form: None,
            _form_sub: None,
        };
        screen.load();
        screen
    }

    fn load(&mut self) {
        self.categories = commands::get_categories(&self.db);
        self.counts = commands::get_transactions(&self.db)
            .map(|t| record_counts(&t))
            .unwrap_or_default();
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load();
        cx.notify();
    }

    pub fn categories(&self) -> &[Category] {
        self.categories.as_deref().unwrap_or(&[])
    }

    #[cfg(test)]
    pub fn has_form(&self) -> bool {
        self.form.is_some()
    }

    pub fn open_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open(None, window, cx);
    }

    fn open_edit(&mut self, category: Category, window: &mut Window, cx: &mut Context<Self>) {
        self.open(Some(category), window, cx);
    }

    fn open(&mut self, editing: Option<Category>, window: &mut Window, cx: &mut Context<Self>) {
        let form = cx.new(|cx| CategoryForm::new(self.db.clone(), editing, window, cx));
        self._form_sub = Some(cx.subscribe(&form, |this, _, event: &FormEvent, cx| {
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
            this.form = None;
            this._form_sub = None;
            cx.notify();
        }));
        self.form = Some(form);
        cx.notify();
    }

    fn delete(&mut self, id: String, cx: &mut Context<Self>) {
        match commands::delete_category(&self.db, id) {
            Ok(()) => self.load(),
            Err(e) => cx.emit(ScreenEvent::error(format!("Could not delete category: {e}"))),
        }
        cx.notify();
    }

    fn section(&self, title: &str, kind: &str, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let rows: Vec<Category> = self
            .categories()
            .iter()
            .filter(|c| c.category_type == kind)
            .cloned()
            .collect();
        let body = if rows.is_empty() {
            div()
                .py(rems(1.))
                .text_color(t.secondary)
                .child(format!("No {kind} categories"))
                .into_any_element()
        } else {
            let mut list = div().flex().flex_col().border_1().border_color(t.border);
            for (i, c) in rows.into_iter().enumerate() {
                let count = self.counts.get(&c.id).copied().unwrap_or(0);
                let color = c.color.as_deref().and_then(parse_hex).unwrap_or(t.secondary);
                let edit = c.clone();
                let id = c.id.clone();
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rems(0.5))
                        .px(rems(0.5))
                        .py(rems(0.25))
                        .when(i > 0, |r| r.border_t_1().border_color(t.border))
                        .child(
                            div()
                                .flex()
                                .flex_1()
                                .items_center()
                                .gap(rems(0.5))
                                .children(c.icon.as_deref().map(|n| category_icon(n, px(14.), color)))
                                .child(c.name.clone()),
                        )
                        .child(
                            div()
                                .text_color(if count > 0 { t.secondary } else { t.foreground.opacity(0.4) })
                                .child(records_label(count)),
                        )
                        .child(
                            icon_button(
                                SharedString::from(format!("edit-cat-{}", c.id)),
                                IconName::Pencil,
                                "Edit category",
                                ButtonVariant::Secondary,
                                cx,
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_edit(edit.clone(), window, cx)
                            })),
                        )
                        .child(
                            icon_button(
                                SharedString::from(format!("del-cat-{}", c.id)),
                                IconName::Trash,
                                "Delete category",
                                ButtonVariant::Danger,
                                cx,
                            )
                            .on_click(cx.listener(move |this, _, _, cx| this.delete(id.clone(), cx))),
                        ),
                );
            }
            list.into_any_element()
        };
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(div().text_size(rems(0.875)).font_weight(gpui_kit::FontWeight::BOLD).child(title.to_string()))
            .child(body)
    }
}

pub fn parse_hex(value: &str) -> Option<Hsla> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok().map(|v| gpui_kit::rgb(v).into())
}

impl Render for CategoriesScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let add = button("add-category", "+ Add", ButtonVariant::Primary, cx)
            .on_click(cx.listener(|this, _, window, cx| this.open_add(window, cx)));
        let expenses = self.section("Expenses", "expense", cx);
        let income = self.section("Income", "income", cx);
        div()
            .id("categories")
            .relative()
            .size_full()
            .overflow_y_scroll()
            .p(rems(1.5))
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(page_header("Categories", add, cx))
            .child(div().flex().gap(rems(1.5)).child(expenses).child(income))
            .children(self.form.clone())
    }
}

pub struct CategoryForm {
    db: Arc<DbState>,
    editing: Option<String>,
    name: Entity<InputState>,
    kind: Entity<ChoiceState>,
    selected_icon: String,
}

impl EventEmitter<FormEvent> for CategoryForm {}

impl CategoryForm {
    fn new(db: Arc<DbState>, editing: Option<Category>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (name, kind, icon) = match &editing {
            Some(c) => (c.name.clone(), c.category_type.clone(), c.icon.clone().unwrap_or_else(|| DEFAULT_ICON.into())),
            None => (String::new(), "expense".into(), DEFAULT_ICON.into()),
        };
        let name_input = text_input_with("e.g. Groceries", &name, window, cx);
        name_input.update(cx, |s, cx| s.focus(window, cx));
        Self {
            db,
            editing: editing.map(|c| c.id),
            name: name_input,
            kind: choice(
                vec![("expense".into(), "Expense".into()), ("income".into(), "Income".into())],
                Some(&kind),
                window,
                cx,
            ),
            selected_icon: icon,
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let name = text_of(&self.name, cx);
        if name.is_empty() {
            return;
        }
        let category_type = choice_value(&self.kind, cx).unwrap_or_else(|| "expense".into());
        let result = match &self.editing {
            None => commands::create_category(
                &self.db,
                CreateCategoryRequest {
                    name: name.clone(),
                    parent_id: None,
                    category_type,
                    icon: Some(self.selected_icon.clone()),
                    color: None,
                },
            )
            .map(|_| format!("Added category {name}")),
            Some(id) => commands::update_category(
                &self.db,
                UpdateCategoryRequest {
                    id: id.clone(),
                    name: Some(name.clone()),
                    parent_id: None,
                    category_type: Some(category_type),
                    icon: Some(Some(self.selected_icon.clone())),
                    color: None,
                },
            )
            .map(|_| format!("Saved category {name}")),
        };
        match result {
            Ok(message) => cx.emit(FormEvent::Saved(message)),
            Err(e) => cx.emit(FormEvent::Failed(format!("Could not save category: {e}"))),
        }
    }
}

impl Render for CategoryForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let picker = div().flex().flex_wrap().gap(rems(0.25)).children(CATEGORY_ICONS.iter().map(|(name, label)| {
            let selected = self.selected_icon == *name;
            let name_owned = name.to_string();
            let fg = if selected { t.accent } else { t.foreground };
            button(
                SharedString::from(format!("pick-icon-{name}")),
                "",
                if selected { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                cx,
            )
            .accessibility_label(*label)
            .p(rems(0.375))
            .child(category_icon(name, px(16.), fg))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected_icon = name_owned.clone();
                cx.notify();
            }))
        }));
        let editing = self.editing.is_some();
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(field("Name", input("category-name", &self.name, window, cx), cx))
            .child(field("Type", select("category-type", &self.kind, window, cx), cx))
            .child(field("Icon", picker, cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(
                        button("category-cancel", "Cancel", ButtonVariant::Outline, cx)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel))),
                    )
                    .child(
                        button("category-submit", if editing { "Save" } else { "Add" }, ButtonVariant::Primary, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                    ),
            );
        let _ = icon;
        modal(
            if editing { "Edit Category" } else { "Add Category" },
            body,
            cx.listener(|_, _, _, cx| cx.emit(FormEvent::Cancel)),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use gpui_kit::{TestAppContext, VisualTestContext};

    fn setup(cx: &mut TestAppContext) -> (Entity<CategoriesScreen>, &mut VisualTestContext) {
        init(cx);
        let db = seeded_db();
        cx.add_window_view(|_, _| CategoriesScreen::new(db))
    }

    fn type_into(input: &Entity<InputState>, text: &str, cx: &mut VisualTestContext) {
        cx.update(|window, cx| input.update(cx, |i, cx| i.set_value(text, window, cx)));
    }

    #[test]
    fn counts_records_per_category() {
        let tx = |cat: Option<&str>| Transaction {
            id: "t".into(),
            account_id: "a".into(),
            category_id: cat.map(String::from),
            transaction_type: "expense".into(),
            amount: 1,
            currency: "USD".into(),
            description: "d".into(),
            merchant: None,
            notes: None,
            transaction_date: "2026-01-01".into(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let counts = record_counts(&[tx(Some("a")), tx(Some("a")), tx(Some("b")), tx(None)]);
        assert_eq!(counts["a"], 2);
        assert_eq!(counts["b"], 1);
        assert_eq!(counts.len(), 2);
        assert_eq!(records_label(1), "1 record");
        assert_eq!(records_label(0), "0 records");
    }

    #[test]
    fn parses_hex_colors() {
        assert!(parse_hex("#7aa2f7").is_some());
        assert!(parse_hex("7aa2f7").is_some());
        assert!(parse_hex("nope").is_none());
        assert!(parse_hex("#fff").is_none());
    }

    #[gpui_kit::test]
    fn lists_seeded_categories_with_counts(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx);
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.categories().len(), 8);
            assert!(s.counts.values().sum::<usize>() > 0, "seeded transactions reference categories");
        });
    }

    #[gpui_kit::test]
    fn add_category_saves(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = cx.update(|_, cx| screen.read(cx).form.clone().expect("form open"));
        let name = cx.update(|_, cx| form.read(cx).name.clone());
        type_into(&name, "Pets", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| {
            f.selected_icon = "dog".into();
            f.submit(cx)
        }));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert!(!s.has_form());
            let added = s.categories().iter().find(|c| c.name == "Pets").expect("saved");
            assert_eq!(added.icon.as_deref(), Some("dog"));
            assert_eq!(added.category_type, "expense");
        });
    }

    #[gpui_kit::test]
    fn edit_category_updates_in_place(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx);
        let original = cx.update(|_, cx| screen.read(cx).categories()[0].clone());
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_edit(original.clone(), window, cx)));
        let form = cx.update(|_, cx| screen.read(cx).form.clone().expect("form open"));
        let name = cx.update(|_, cx| form.read(cx).name.clone());
        cx.update(|_, cx| assert_eq!(name.read(cx).value().as_ref(), original.name));
        type_into(&name, "Renamed", cx);
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            let s = screen.read(cx);
            assert_eq!(s.categories().len(), 8, "no duplicate created");
            assert_eq!(s.categories().iter().filter(|c| c.id == original.id).next().unwrap().name, "Renamed");
        });
    }

    #[gpui_kit::test]
    fn blank_name_keeps_form_open(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx);
        cx.update(|window, cx| screen.update(cx, |s, cx| s.open_add(window, cx)));
        let form = cx.update(|_, cx| screen.read(cx).form.clone().unwrap());
        cx.update(|_, cx| form.update(cx, |f, cx| f.submit(cx)));
        cx.update(|_, cx| {
            assert!(screen.read(cx).has_form());
            assert_eq!(screen.read(cx).categories().len(), 8);
        });
    }

    #[gpui_kit::test]
    fn delete_removes_category(cx: &mut TestAppContext) {
        let (screen, cx) = setup(cx);
        let id = cx.update(|_, cx| screen.read(cx).categories()[0].id.clone());
        cx.update(|_, cx| screen.update(cx, |s, cx| s.delete(id.clone(), cx)));
        cx.update(|_, cx| assert!(screen.read(cx).categories().iter().all(|c| c.id != id)));
    }
}
