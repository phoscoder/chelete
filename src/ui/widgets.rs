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
) -> impl IntoElement {
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
                .w(rems(26.))
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
