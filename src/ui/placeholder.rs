use super::View;
use gpui_kit::{div, prelude::*, rems, App, IntoElement};
use gpui_omarchy::{empty_state, panel, ActiveTheme as _};

/// Stand-in for screens that have not been ported from the web UI yet.
pub fn placeholder(view: View, cx: &App) -> impl IntoElement {
    let t = cx.omarchy();
    div().size_full().p(rems(1.5)).child(
        panel(view.label(), cx).child(
            div()
                .text_color(t.secondary)
                .child(empty_state("Coming soon", "This screen has not been ported yet.", cx)),
        ),
    )
}
