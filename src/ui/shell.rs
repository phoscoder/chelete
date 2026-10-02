use super::overview::{self, OverviewData};
use super::palette::{Command, Palette, PaletteEvent};
use super::placeholder::placeholder;
use super::sidebar::sidebar;
use super::*;
use chelete_lib::database::DbState;
use chelete_lib::prefs::Prefs;
use gpui_kit::{
    div, prelude::*, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Subscription,
    Window,
};
use gpui_omarchy::ActiveTheme as _;
use std::sync::Arc;

pub struct Shell {
    db: Arc<DbState>,
    view: View,
    prefs: Prefs,
    persist: bool,
    overview: OverviewData,
    palette: Option<Entity<Palette>>,
    focus: FocusHandle,
    _palette_sub: Option<Subscription>,
}

impl Shell {
    pub fn new(
        db: Arc<DbState>,
        prefs: Prefs,
        persist: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            overview: OverviewData::load(&db),
            db,
            view: View::Overview,
            prefs,
            persist,
            palette: None,
            focus,
            _palette_sub: None,
        }
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn collapsed(&self) -> bool {
        self.prefs.sidebar_collapsed
    }

    pub fn navigate(&mut self, view: View, window: &mut Window, cx: &mut Context<Self>) {
        self.close_palette(window, cx);
        self.view = view;
        if view == View::Overview {
            self.overview = OverviewData::load(&self.db);
        }
        cx.notify();
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.prefs.sidebar_collapsed = !self.prefs.sidebar_collapsed;
        if self.persist {
            self.prefs.save();
        }
        cx.notify();
    }

    fn toggle_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.is_some() {
            self.close_palette(window, cx);
            return;
        }
        let palette = cx.new(|cx| Palette::new(window, cx));
        self._palette_sub = Some(cx.subscribe_in(
            &palette,
            window,
            |this, _, event: &PaletteEvent, window, cx| match event {
                PaletteEvent::Dismiss => this.close_palette(window, cx),
                PaletteEvent::Run(command) => this.run(*command, window, cx),
            },
        ));
        self.palette = Some(palette);
        cx.notify();
    }

    fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette.take().is_some() {
            self._palette_sub = None;
            self.focus.focus(window, cx);
            cx.notify();
        }
    }

    /// Creation flows live on their own screens; until those are ported the
    /// "Add …" commands just take you there.
    fn run(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        let target = match command {
            Command::Go(view) => view,
            Command::AddTransaction | Command::ImportTransactions => View::Transactions,
            Command::AddAccount => View::Accounts,
            Command::AddCategory => View::Categories,
            Command::AddSubscription => View::Subscriptions,
        };
        self.navigate(target, window, cx);
    }
}

impl Focusable for Shell {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = cx.omarchy().clone();
        let content = match self.view {
            View::Overview => overview::render(&self.overview, cx).into_any_element(),
            other => placeholder(other, cx).into_any_element(),
        };
        div()
            .key_context(SHELL_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &GoOverview, w, cx| this.navigate(View::Overview, w, cx)))
            .on_action(cx.listener(|this, _: &GoTransactions, w, cx| this.navigate(View::Transactions, w, cx)))
            .on_action(cx.listener(|this, _: &GoAccounts, w, cx| this.navigate(View::Accounts, w, cx)))
            .on_action(cx.listener(|this, _: &GoCategories, w, cx| this.navigate(View::Categories, w, cx)))
            .on_action(cx.listener(|this, _: &GoSubscriptions, w, cx| this.navigate(View::Subscriptions, w, cx)))
            .on_action(cx.listener(|this, _: &GoProjections, w, cx| this.navigate(View::Projections, w, cx)))
            .on_action(cx.listener(|this, _: &GoSettings, w, cx| this.navigate(View::Settings, w, cx)))
            .on_action(cx.listener(|this, _: &TogglePalette, w, cx| this.toggle_palette(w, cx)))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| this.toggle_sidebar(cx)))
            .size_full()
            .relative()
            .flex()
            .bg(t.background)
            .text_color(t.foreground)
            .font_family(t.font.clone())
            .child(sidebar(self, cx))
            .child(div().flex_1().min_w_0().h_full().child(content))
            .when_some(self.palette.clone(), |el, palette| el.child(palette))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chelete_lib::{database, seed};
    use gpui_kit::TestAppContext;
    use rusqlite::Connection;

    fn seeded_db() -> Arc<DbState> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        database::run_migrations(&conn).unwrap();
        seed::seed_database(&conn).unwrap();
        Arc::new(DbState(std::sync::Mutex::new(conn)))
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<Shell>, &mut gpui_kit::VisualTestContext) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_omarchy::init(cx);
            // Applying a fixed theme stops the filesystem watcher, which the
            // deterministic test scheduler rejects.
            gpui_omarchy::Theme::tokyo_night().apply(cx);
            bind_keys(cx);
        });
        let db = seeded_db();
        cx.add_window_view(|window, cx| Shell::new(db, Prefs::default(), false, window, cx))
    }

    fn view_of(shell: &Entity<Shell>, cx: &mut gpui_kit::VisualTestContext) -> View {
        cx.update(|_, cx| shell.read(cx).view())
    }

    #[gpui_kit::test]
    fn starts_on_overview_with_seeded_data(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.update(|_, cx| {
            let shell = shell.read(cx);
            assert_eq!(shell.view(), View::Overview);
            let overview = shell.overview.result.as_ref().expect("overview loads");
            assert_eq!(overview.accounts.len(), 4);
        });
    }

    #[gpui_kit::test]
    fn ctrl_shortcuts_switch_views(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        for (keys, expected) in [
            ("ctrl-t", View::Transactions),
            ("ctrl-a", View::Accounts),
            ("ctrl-c", View::Categories),
            ("ctrl-u", View::Subscriptions),
            ("ctrl-p", View::Projections),
            ("ctrl-s", View::Settings),
            ("ctrl-o", View::Overview),
        ] {
            cx.simulate_keystrokes(keys);
            assert_eq!(view_of(&shell, cx), expected, "after {keys}");
        }
    }

    #[gpui_kit::test]
    fn ctrl_b_toggles_sidebar(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-b");
        cx.update(|_, cx| assert!(shell.read(cx).collapsed()));
        cx.simulate_keystrokes("ctrl-b");
        cx.update(|_, cx| assert!(!shell.read(cx).collapsed()));
    }

    #[gpui_kit::test]
    fn palette_opens_filters_and_navigates(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-k");
        cx.update(|_, cx| assert!(shell.read(cx).palette.is_some(), "palette opens"));

        cx.simulate_input("go to acc");
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| {
            assert!(shell.read(cx).palette.is_none(), "palette closes after running");
        });
        assert_eq!(view_of(&shell, cx), View::Accounts);
    }

    #[gpui_kit::test]
    fn palette_escape_closes_without_navigating(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-k");
        cx.simulate_keystrokes("escape");
        cx.update(|_, cx| assert!(shell.read(cx).palette.is_none()));
        assert_eq!(view_of(&shell, cx), View::Overview);
    }

    #[gpui_kit::test]
    fn palette_arrow_keys_pick_the_command(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-k");
        cx.simulate_keystrokes("down down enter");
        // Entries are ordered like View::ALL: Overview, Transactions, Accounts.
        assert_eq!(view_of(&shell, cx), View::Accounts);
    }
}
