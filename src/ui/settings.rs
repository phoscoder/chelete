use super::widgets::{element_tile, page_header, TileDetail};
use gpui_kit::{div, prelude::*, rems, App, Div, IntoElement};
use gpui_omarchy::{panel, separator, ActiveTheme as _};

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

pub fn render(cx: &App) -> impl IntoElement {
    let t = cx.omarchy();
    let theme_name = t.name.to_string();

    let appearance = panel("Appearance", cx)
        .child(row("Theme", Some("Follows the Omarchy system theme"), theme_name, cx))
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
