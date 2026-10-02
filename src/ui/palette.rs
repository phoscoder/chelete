use super::{PaletteDismiss, PaletteNext, PalettePrev, View, PALETTE_CONTEXT};
use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::{
    div, prelude::*, rems, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, MouseButton, Render, SharedString, Subscription, Window,
};
use gpui_omarchy::{input, keycap, ActiveTheme as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Go(View),
    AddTransaction,
    ImportTransactions,
    AddAccount,
    AddCategory,
    AddSubscription,
}

pub enum PaletteEvent {
    Run(Command),
    Dismiss,
}

struct Entry {
    label: SharedString,
    shortcut: Option<&'static str>,
    command: Command,
}

fn entries() -> Vec<Entry> {
    let mut all: Vec<Entry> = View::ALL
        .iter()
        .map(|v| Entry {
            label: format!("Go to {}", v.label()).into(),
            shortcut: Some(v.shortcut()),
            command: Command::Go(*v),
        })
        .collect();
    for (label, command) in [
        ("Add Transaction", Command::AddTransaction),
        ("Import Transactions (CSV)", Command::ImportTransactions),
        ("Add Account", Command::AddAccount),
        ("Add Category", Command::AddCategory),
        ("Add Subscription", Command::AddSubscription),
    ] {
        all.push(Entry {
            label: label.into(),
            shortcut: None,
            command,
        });
    }
    all
}

pub struct Palette {
    query: Entity<InputState>,
    selected: usize,
    focus: FocusHandle,
    _subs: Vec<Subscription>,
}

impl EventEmitter<PaletteEvent> for Palette {}

impl Focusable for Palette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Palette {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Type a command..."));
        let sub = cx.subscribe_in(&query, window, |this, _, event: &InputEvent, _, cx| match event {
            InputEvent::Change => {
                this.selected = 0;
                cx.notify();
            }
            InputEvent::PressEnter { .. } => this.confirm(cx),
            _ => {}
        });
        query.update(cx, |state, cx| state.focus(window, cx));
        Self {
            query,
            selected: 0,
            focus: cx.focus_handle(),
            _subs: vec![sub],
        }
    }

    fn matches(&self, cx: &App) -> Vec<Entry> {
        let needle = self.query.read(cx).value().to_lowercase();
        entries()
            .into_iter()
            .filter(|e| e.label.to_lowercase().contains(&needle))
            .collect()
    }

    fn confirm(&mut self, cx: &mut Context<Self>) {
        let matches = self.matches(cx);
        if let Some(entry) = matches.get(self.selected) {
            cx.emit(PaletteEvent::Run(entry.command));
        }
    }

    fn step(&mut self, delta: isize, cx: &mut Context<Self>) {
        let len = self.matches(cx).len();
        if len > 0 {
            self.selected = (self.selected as isize + delta).clamp(0, len as isize - 1) as usize;
            cx.notify();
        }
    }
}

impl Render for Palette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let matches = self.matches(cx);
        let selected = self.selected.min(matches.len().saturating_sub(1));
        let empty = matches.is_empty();
        let query = input("palette-input", &self.query, window, cx);

        let rows = matches.into_iter().enumerate().map(|(i, entry)| {
            let command = entry.command;
            let mut row = div()
                .id(("palette-row", i))
                .flex()
                .items_center()
                .justify_between()
                .px(rems(0.75))
                .py(rems(0.375))
                .cursor_pointer()
                .child(entry.label)
                .on_mouse_move(cx.listener(move |this, _, _, cx| {
                    if this.selected != i {
                        this.selected = i;
                        cx.notify();
                    }
                }))
                .on_click(cx.listener(move |_, _, _, cx| cx.emit(PaletteEvent::Run(command))));
            if i == selected {
                row = row.bg(t.selected_fill());
            }
            if let Some(shortcut) = entry.shortcut {
                row = row.child(keycap(shortcut, cx));
            }
            row
        });

        div()
            .id("palette-backdrop")
            .absolute()
            .size_full()
            .flex()
            .justify_center()
            .pt(rems(5.))
            .bg(t.background.opacity(0.6))
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _, _, cx| cx.emit(PaletteEvent::Dismiss)),
            )
            .child(
                div()
                    .key_context(PALETTE_CONTEXT)
                    .track_focus(&self.focus)
                    .on_action(cx.listener(|this, _: &PaletteNext, _, cx| this.step(1, cx)))
                    .on_action(cx.listener(|this, _: &PalettePrev, _, cx| this.step(-1, cx)))
                    .on_action(cx.listener(|_, _: &PaletteDismiss, _, cx| {
                        cx.emit(PaletteEvent::Dismiss)
                    }))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .w(rems(34.))
                    .max_h(rems(24.))
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(t.border)
                    .bg(t.background)
                    .font_family(t.font.clone())
                    .text_size(rems(0.75))
                    .text_color(t.foreground)
                    .child(div().p(rems(0.5)).child(query))
                    .child(
                        div()
                            .id("palette-results")
                            .flex()
                            .flex_col()
                            .overflow_y_scroll()
                            .children(rows)
                            .when(empty, |el| {
                                el.child(
                                    div()
                                        .p(rems(1.))
                                        .text_color(t.secondary)
                                        .child("No matching commands"),
                                )
                            }),
                    ),
            )
    }
}
