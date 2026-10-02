//! gpui-omarchy themes its own controls; gpui-component widgets (the date
//! picker) read a separate theme. Copy the Omarchy palette across and keep it
//! in step when the system theme changes.
//!
//! It also swaps the UI font. gpui-omarchy defaults to the platform UI font,
//! which resolves to Adwaita Sans on GNOME-flavoured systems, whereas Omarchy
//! itself uses the one font chosen with `omarchy font set`. Use that everywhere.
use gpui_kit::component::Theme as ComponentTheme;
use gpui_kit::{App, BorrowAppContext as _};
use gpui_omarchy::ActiveTheme as _;

/// Make the Omarchy font the UI font for both control libraries.
fn use_omarchy_font(cx: &mut App) {
    let mono = cx.omarchy().mono_font.clone();
    if cx.omarchy().font != mono {
        // Observers see this change too, but by then the fonts already match.
        cx.update_global::<gpui_omarchy::Theme, _>(|theme, _| theme.font = mono.clone());
    }
    // gpui-omarchy copies its UI font into the base theme whenever it applies a palette.
    gpui_kit::base::Theme::global_mut(cx).tokens.typography.sans = mono.clone();
    let t = ComponentTheme::global_mut(cx);
    t.font_family = mono.clone();
    t.mono_font_family = mono;
}

pub fn sync(cx: &mut App) {
    use_omarchy_font(cx);
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

/// Set the size of one rem for the whole app from a text size percentage.
///
/// The window's root re-applies `ComponentTheme.font_size` as the rem size on
/// every frame, so that value has to carry the setting; setting only the
/// window's own rem size is undone on the next frame.
pub fn set_text_scale(percent: u16, window: &mut gpui_kit::Window, cx: &mut App) {
    let rem = gpui_kit::px(chelete_lib::prefs::rem_size_px(percent));
    ComponentTheme::global_mut(cx).font_size = rem;
    window.set_rem_size(rem);
    window.refresh();
}

/// Sync now and again whenever the Omarchy theme changes.
pub fn install(cx: &mut App) {
    sync(cx);
    cx.observe_global::<gpui_omarchy::Theme>(sync).detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::init;
    use gpui_kit::TestAppContext;

    #[gpui_kit::test]
    fn the_ui_uses_the_omarchy_font_even_after_a_theme_change(cx: &mut TestAppContext) {
        init(cx);
        cx.update(|cx| {
            let omarchy = cx.omarchy().clone();
            assert_ne!(omarchy.mono_font.as_ref(), ".SystemUIFont");
            assert_eq!(omarchy.font, omarchy.mono_font, "omarchy controls use the mono font");
            assert_eq!(ComponentTheme::global(cx).font_family, omarchy.mono_font, "so do gpui-component widgets");
            assert_eq!(gpui_kit::base::Theme::global(cx).tokens.typography.sans, omarchy.mono_font);
        });
        // Applying a palette resets the font to the platform UI font; it must be put back.
        cx.update(|cx| gpui_omarchy::Theme::flexoki_light().apply(cx));
        cx.update(|cx| {
            let omarchy = cx.omarchy().clone();
            assert_eq!(omarchy.font, omarchy.mono_font);
            assert_eq!(ComponentTheme::global(cx).font_family, omarchy.mono_font);
            assert_eq!(gpui_kit::base::Theme::global(cx).tokens.typography.sans, omarchy.mono_font);
        });
    }
}
