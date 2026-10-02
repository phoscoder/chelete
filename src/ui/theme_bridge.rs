//! gpui-omarchy themes its own controls; gpui-component widgets (the date
//! picker) read a separate theme. Copy the Omarchy palette across and keep it
//! in step when the system theme changes.
use gpui_kit::component::Theme as ComponentTheme;
use gpui_kit::App;
use gpui_omarchy::ActiveTheme as _;

pub fn sync(cx: &mut App) {
    let o = cx.omarchy().clone();
    let t = ComponentTheme::global_mut(cx);
    t.background = o.background;
    t.foreground = o.foreground;
    t.popover = o.surface;
    t.popover_foreground = o.foreground;
    t.primary = o.accent;
    t.primary_foreground = o.on_accent;
    t.accent = o.selection;
    t.accent_foreground = o.foreground;
    t.muted = o.surface;
    t.muted_foreground = o.secondary;
    t.border = o.border;
    t.input = o.border;
    t.ring = o.accent;
    t.danger = o.danger;
    t.chart_1 = o.chart[0];
    t.chart_2 = o.chart[1];
    t.chart_3 = o.chart[2];
    t.chart_4 = o.chart[3];
    t.chart_5 = o.chart[4];
}

/// Sync now and again whenever the Omarchy theme changes.
pub fn install(cx: &mut App) {
    sync(cx);
    cx.observe_global::<gpui_omarchy::Theme>(sync).detach();
}
