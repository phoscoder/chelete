use super::widgets::{choice, choice_value, element_tile, page_header, TileDetail};
use chelete_lib::prefs::TEXT_SCALES;
use gpui_kit::{div, prelude::*, rems, App, Context, Div, Entity, EventEmitter, IntoElement, Render, Subscription, Window};
use gpui_omarchy::{panel, select, separator, ActiveTheme as _, ChoiceState};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn row(label: &str, description: Option<&str>, value: impl IntoElement, cx: &App) -> Div {
    let t = cx.omarchy();
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .child(label.to_string())
                .children(description.map(|d| div().text_color(t.secondary).child(d.to_string()))),
        )
        .child(div().text_color(t.secondary).child(value))
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsEvent {
    TextScale(u16),
}

pub struct SettingsScreen {
    scale: Entity<ChoiceState>,
    /// The size the dropdown currently shows, so showing a size chosen
    /// elsewhere (the keyboard) is not mistaken for a new choice.
    shown: u16,
    _sub: Subscription,
}

impl EventEmitter<SettingsEvent> for SettingsScreen {}

fn scale_items() -> Vec<(String, String)> {
    TEXT_SCALES.iter().map(|s| (s.to_string(), format!("{s}%"))).collect()
}

impl SettingsScreen {
    pub fn new(text_scale: u16, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let scale = choice(scale_items(), Some(&text_scale.to_string()), window, cx);
        let sub = cx.observe(&scale, |this, state, cx| {
            if let Some(percent) = choice_value(&state, cx).and_then(|v| v.parse::<u16>().ok()) {
                if percent != this.shown {
                    this.shown = percent;
                    cx.emit(SettingsEvent::TextScale(percent));
                }
            }
        });
        Self { scale, shown: text_scale, _sub: sub }
    }

    /// Reflect a size that was changed elsewhere (a keyboard shortcut).
    pub fn show_scale(&mut self, percent: u16, cx: &mut Context<Self>) {
        if self.shown == percent {
            return;
        }
        self.shown = percent;
        let index = TEXT_SCALES.iter().position(|s| *s == percent);
        self.scale.update(cx, |state, cx| state.set_selected(index, cx));
        cx.notify();
    }

    #[cfg(test)]
    pub fn shown(&self) -> u16 {
        self.shown
    }

    #[cfg(test)]
    pub fn choose(&mut self, percent: u16, cx: &mut Context<Self>) {
        let index = TEXT_SCALES.iter().position(|s| *s == percent);
        self.scale.update(cx, |state, cx| state.set_selected(index, cx));
    }
}

impl Render for SettingsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_page(&self.scale, window, cx)
    }
}

fn render_page(scale: &Entity<ChoiceState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let t = cx.omarchy().clone();
    let theme_name = t.name.to_string();

    let appearance = panel("Appearance", cx)
        .child(row("Theme", Some("Follows the Omarchy system theme"), theme_name, cx))
        .child(separator(cx))
        .child(separator(cx))
        .child(row("Text size", Some("Ctrl + larger, Ctrl - smaller, Ctrl 0 reset"), div().w(rems(7.)).child(select("text-scale", scale, window, cx)), cx))
        .child(separator(cx))
        .child(row("Corners", Some("Square, like the rest of Omarchy"), "Square", cx));

    let finance = panel("Finance", cx)
        .child(row("Base Currency", None, "USD", cx))
        .child(separator(cx))
        .child(row("Month Start", None, "1st", cx));

    let about = panel("About", cx)
        .child(
            div()
                .flex()
                .items_center()
                .gap(rems(0.75))
                .child(element_tile(4.5, TileDetail::Full, cx))
                .child(
                    div()
                        .child(div().text_size(rems(1.)).font_weight(gpui_kit::FontWeight::BOLD).child("Chelete"))
                        .child(div().text_color(t.secondary).child(format!("Version {VERSION}"))),
                ),
        )
        .child(div().text_color(t.secondary).child(
            "A fast, keyboard-driven personal finance tracker for Linux. Manage accounts, \
             track transactions, and project your balance over time — without touching the \
             mouse. Built for Omarchy with live theme integration.",
        ))
        .child(separator(cx))
        .child(row("Author", None, "Victor Phos", cx))
        .child(separator(cx))
        .child(row("License", None, "MIT", cx));

    div()
        .id("settings")
        .size_full()
        .overflow_y_scroll()
        .p(rems(1.5))
        .flex()
        .flex_col()
        .gap(rems(1.))
        .child(page_header("Settings", div(), cx))
        .child(div().max_w(rems(36.)).flex().flex_col().gap(rems(1.)).child(appearance).child(finance).child(about))
}
