//! Small building blocks shared by the screens: page headers, form fields,
//! modals, inputs and choice lists.
use gpui_kit::base::input::InputState;
use gpui_kit::{
    div, prelude::*, rems, App, Context, Div, Entity, FontWeight, IntoElement, MouseButton,
    MouseDownEvent, SharedString, Window,
};
use gpui_omarchy::{ActiveTheme as _, ChoiceItem, ChoiceState};

/// Emitted by screens and forms so the shell can show a toast.
#[derive(Clone, Debug)]
pub enum ScreenEvent {
    Toast { message: String, ok: bool },
}

impl ScreenEvent {
    pub fn ok(message: impl Into<String>) -> Self {
        Self::Toast { message: message.into(), ok: true }
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self::Toast { message: message.into(), ok: false }
    }
}

pub fn page_header(title: &str, actions: impl IntoElement, cx: &App) -> Div {
    let t = cx.omarchy();
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(rems(1.25))
                .font_weight(FontWeight::BOLD)
                .text_color(t.foreground)
                .child(title.to_string()),
        )
        .child(div().flex().gap(rems(0.5)).child(actions))
}

pub fn field(label: &str, body: impl IntoElement, cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap(rems(0.25))
        .child(div().text_color(cx.omarchy().secondary).child(label.to_string()))
        .child(body)
}

/// A centered dialog over a dimmed, click-to-dismiss backdrop. Place it as the
/// last child of a `relative` container.
pub fn modal(
    title: &str,
    body: impl IntoElement,
    on_dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> gpui_kit::Stateful<Div> {
    modal_sized(26., title, body, on_dismiss, cx)
}

/// Like [`modal`] with an explicit width in rems.
pub fn modal_sized(
    width: f32,
    title: &str,
    body: impl IntoElement,
    on_dismiss: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> gpui_kit::Stateful<Div> {
    let t = cx.omarchy();
    div()
        .id("modal-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(t.background.opacity(0.6))
        .occlude()
        .on_mouse_down(MouseButton::Left, on_dismiss)
        .child(
            div()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .w(rems(width))
                .max_w_full()
                .max_h_full()
                .flex()
                .flex_col()
                .gap(rems(0.875))
                .p(rems(1.125))
                .border_1()
                .border_color(t.border)
                .bg(t.background)
                .font_family(t.font.clone())
                .text_size(rems(0.75))
                .text_color(t.foreground)
                .child(
                    div()
                        .text_size(rems(1.))
                        .font_weight(FontWeight::BOLD)
                        .child(title.to_string()),
                )
                .child(body),
        )
}

pub fn text_input<V: 'static>(
    placeholder: &str,
    window: &mut Window,
    cx: &mut Context<V>,
) -> Entity<InputState> {
    let placeholder = placeholder.to_string();
    cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
}

pub fn text_input_with<V: 'static>(
    placeholder: &str,
    value: &str,
    window: &mut Window,
    cx: &mut Context<V>,
) -> Entity<InputState> {
    let placeholder = placeholder.to_string();
    let value = value.to_string();
    cx.new(|cx| InputState::new(window, cx).placeholder(placeholder).default_value(value))
}

pub fn text_of(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().trim().to_string()
}

/// A select over `(value, label)` pairs with `default` pre-selected when it is
/// one of the values.
pub fn choice<V: 'static>(
    items: Vec<(String, String)>,
    default: Option<&str>,
    window: &mut Window,
    cx: &mut Context<V>,
) -> Entity<ChoiceState> {
    let selected = default.and_then(|d| items.iter().position(|(v, _)| v == d));
    let items: Vec<ChoiceItem> = items
        .into_iter()
        .map(|(v, l)| ChoiceItem::new(SharedString::from(v), SharedString::from(l)))
        .collect();
    cx.new(|cx| {
        let state = ChoiceState::new(items, window, cx);
        match selected {
            Some(i) => state.default_selected(i),
            None => state,
        }
    })
}

pub fn choice_value(state: &Entity<ChoiceState>, cx: &App) -> Option<String> {
    state.read(cx).selected().map(|i| i.value.to_string())
}

/// The standard account types, in the order the web UI listed them.
pub fn account_type_options() -> Vec<(String, String)> {
    [
        ("cash", "Cash"),
        ("bank", "Bank"),
        ("savings", "Savings"),
        ("credit_card", "Credit Card"),
        ("mobile_money", "Mobile Money"),
        ("investment", "Investment"),
        ("other", "Other"),
    ]
    .into_iter()
    .map(|(v, l)| (v.to_string(), l.to_string()))
    .collect()
}

// ── lists ───────────────────────────────────────────────────────────

use chelete_lib::paging::{Pager, PER_PAGE_OPTIONS};
use gpui_kit::base::CheckboxState;
use gpui_kit::{ClickEvent, EventEmitter};
use gpui_kit::base::Checkbox;
use gpui_omarchy::{button, checkbox, select, ButtonVariant};

pub fn check(
    id: impl Into<gpui_kit::ElementId>,
    label: &str,
    state: CheckboxState,
    on_toggle: impl Fn(&mut Window, &mut App) + 'static,
    cx: &App,
) -> Checkbox {
    checkbox(id, label.to_string(), state, cx)
        .on_change(move |_, _: &ClickEvent, window, cx| on_toggle(window, cx))
}

pub fn per_page_choice<V: 'static>(current: usize, window: &mut Window, cx: &mut Context<V>) -> Entity<ChoiceState> {
    choice(
        PER_PAGE_OPTIONS.iter().map(|n| (n.to_string(), n.to_string())).collect(),
        Some(&current.to_string()),
        window,
        cx,
    )
}

/// "Show [25] per page            Previous  1–25 of 61  Next"
pub fn pagination_bar(
    pager: Pager,
    per_page: &Entity<ChoiceState>,
    on_prev: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_next: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let t = cx.omarchy().clone();
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(rems(0.5))
                .child(div().text_color(t.secondary).child("Show"))
                .child(div().w(rems(5.)).child(select("per-page", per_page, window, cx)))
                .child(div().text_color(t.secondary).child("per page")),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(rems(0.75))
                .child(
                    button("page-prev", "Previous", ButtonVariant::Outline, cx)
                        .disabled(!pager.has_prev())
                        .on_click(on_prev),
                )
                .child(div().text_color(t.secondary).child(pager.label()))
                .child(
                    button("page-next", "Next", ButtonVariant::Outline, cx)
                        .disabled(!pager.has_next())
                        .on_click(on_next),
                ),
        )
}

pub fn selection_state(selected_on_page: usize, page_len: usize) -> CheckboxState {
    if page_len > 0 && selected_on_page == page_len {
        CheckboxState::Checked
    } else if selected_on_page > 0 {
        CheckboxState::Indeterminate
    } else {
        CheckboxState::Unchecked
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmEvent {
    Confirm,
    Cancel,
}

/// A yes/no dialog for destructive actions.
pub struct ConfirmDialog {
    title: SharedString,
    message: SharedString,
    confirm_label: SharedString,
}

impl EventEmitter<ConfirmEvent> for ConfirmDialog {}

impl ConfirmDialog {
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>, confirm_label: impl Into<SharedString>) -> Self {
        Self { title: title.into(), message: message.into(), confirm_label: confirm_label.into() }
    }
}

impl Render for ConfirmDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = div()
            .flex()
            .flex_col()
            .gap(rems(1.))
            .child(div().text_color(cx.omarchy().secondary).child(self.message.clone()))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(rems(0.5))
                    .child(
                        button("confirm-cancel", "Cancel", ButtonVariant::Outline, cx)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ConfirmEvent::Cancel))),
                    )
                    .child(
                        button("confirm-ok", self.confirm_label.clone(), ButtonVariant::Danger, cx)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(ConfirmEvent::Confirm))),
                    ),
            );
        modal(&self.title, body, cx.listener(|_, _, _, cx| cx.emit(ConfirmEvent::Cancel)), cx)
    }
}

pub fn plural(count: usize, singular: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {singular}s")
    }
}

pub fn dim(text: impl Into<SharedString>, cx: &App) -> Div {
    div().text_color(cx.omarchy().secondary).child(text.into())
}

/// A table cell: fixed width in rems, or flexible when `None`.
pub fn cell(width: Option<f32>, child: impl IntoElement) -> Div {
    let base = div().min_w_0().px(rems(0.5));
    match width {
        Some(w) => base.w(rems(w)).flex_shrink_0().child(child),
        None => base.flex_1().child(child),
    }
}

/// Previous / range / Next, for lists with a fixed page size.
pub fn simple_pager(
    pager: Pager,
    on_prev: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_next: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Div {
    div()
        .flex()
        .items_center()
        .justify_end()
        .gap(rems(0.75))
        .child(button("pager-prev", "Previous", ButtonVariant::Outline, cx).disabled(!pager.has_prev()).on_click(on_prev))
        .child(dim(pager.label(), cx))
        .child(button("pager-next", "Next", ButtonVariant::Outline, cx).disabled(!pager.has_next()).on_click(on_next))
}
