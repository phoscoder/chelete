//! GPUI front end: app shell, sidebar, command palette and screens.
mod accounts;
mod categories;
mod csv_import;
mod date_field;
mod date_filter_control;
mod icons;
mod overview;
mod palette;
mod projections;
mod shell;
mod settings;
mod sidebar;
mod subscriptions;
mod theme_bridge;
mod transactions;
mod widgets;

#[cfg(test)]
mod test_support;

pub use shell::Shell;

use gpui_kit::{actions, App, KeyBinding};
use gpui_omarchy::IconName;

actions!(
    chelete,
    [
        GoOverview,
        GoTransactions,
        GoAccounts,
        GoCategories,
        GoSubscriptions,
        GoProjections,
        GoSettings,
        TogglePalette,
        ToggleSidebar,
        Quit,
        PaletteNext,
        PalettePrev,
        PaletteDismiss,
        ModalCancel,
        ZoomIn,
        ZoomOut,
        ZoomReset,
    ]
);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Overview,
    Transactions,
    Accounts,
    Categories,
    Subscriptions,
    Projections,
    Settings,
}

impl View {
    pub const ALL: [View; 7] = [
        View::Overview,
        View::Transactions,
        View::Accounts,
        View::Categories,
        View::Subscriptions,
        View::Projections,
        View::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            View::Overview => "Overview",
            View::Transactions => "Transactions",
            View::Accounts => "Accounts",
            View::Categories => "Categories",
            View::Subscriptions => "Subscriptions",
            View::Projections => "Projections",
            View::Settings => "Settings",
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            View::Overview => "Ctrl+O",
            View::Transactions => "Ctrl+T",
            View::Accounts => "Ctrl+A",
            View::Categories => "Ctrl+C",
            View::Subscriptions => "Ctrl+U",
            View::Projections => "Ctrl+P",
            View::Settings => "Ctrl+S",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            View::Overview => IconName::LayoutDashboard,
            View::Transactions => IconName::ArrowLeftRight,
            View::Accounts => IconName::Wallet,
            View::Categories => IconName::Tag,
            View::Subscriptions => IconName::Repeat,
            View::Projections => IconName::ChartLine,
            View::Settings => IconName::Settings,
        }
    }
}

/// Context the shell sets on its root element; shortcuts below only fire inside it,
/// so text inputs keep their own Ctrl+A / Ctrl+C / Ctrl+S behavior.
pub const SHELL_CONTEXT: &str = "Shell";
pub const PALETTE_CONTEXT: &str = "Palette";
pub const MODAL_CONTEXT: &str = "Modal";

pub fn install(cx: &mut App) {
    bind_keys(cx);
    theme_bridge::install(cx);
}

pub fn bind_keys(cx: &mut App) {
    let shell = Some(SHELL_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("ctrl-o", GoOverview, shell),
        KeyBinding::new("ctrl-t", GoTransactions, shell),
        KeyBinding::new("ctrl-a", GoAccounts, shell),
        KeyBinding::new("ctrl-c", GoCategories, shell),
        KeyBinding::new("ctrl-u", GoSubscriptions, shell),
        KeyBinding::new("ctrl-p", GoProjections, shell),
        KeyBinding::new("ctrl-s", GoSettings, shell),
        KeyBinding::new("ctrl-k", TogglePalette, shell),
        KeyBinding::new("ctrl-b", ToggleSidebar, shell),
        KeyBinding::new("ctrl-=", ZoomIn, shell),
        KeyBinding::new("ctrl-+", ZoomIn, shell),
        KeyBinding::new("ctrl--", ZoomOut, shell),
        KeyBinding::new("ctrl-0", ZoomReset, shell),
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("down", PaletteNext, Some(PALETTE_CONTEXT)),
        KeyBinding::new("up", PalettePrev, Some(PALETTE_CONTEXT)),
        KeyBinding::new("escape", PaletteDismiss, Some(PALETTE_CONTEXT)),
        KeyBinding::new("escape", ModalCancel, Some(MODAL_CONTEXT)),
    ]);
}
