use super::accounts::AccountsScreen;
use super::categories::CategoriesScreen;
use super::projections::ProjectionsScreen;
use super::settings::{SettingsEvent, SettingsScreen};
use super::subscriptions::SubscriptionsScreen;
use super::transactions::TransactionsScreen;
use super::overview::OverviewScreen;
use super::widgets::ScreenEvent;
use super::palette::{Command, Palette, PaletteEvent};
use super::sidebar::sidebar;
use super::*;
use chelete_lib::database::DbState;
use chelete_lib::prefs::{clamp_text_scale, step_text_scale, Prefs, DEFAULT_TEXT_SCALE};
use gpui_kit::{
    div, prelude::*, rems, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Subscription,
    Window,
};
use gpui_omarchy::ActiveTheme as _;
use std::sync::Arc;

pub struct Shell {
    view: View,
    prefs: Prefs,
    persist: bool,
    overview: Entity<OverviewScreen>,
    accounts: Entity<AccountsScreen>,
    categories: Entity<CategoriesScreen>,
    subscriptions: Entity<SubscriptionsScreen>,
    projections: Entity<ProjectionsScreen>,
    transactions: Entity<TransactionsScreen>,
    settings: Entity<SettingsScreen>,
    toast: Option<(u64, String, bool)>,
    toast_seq: u64,
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
        let accounts = cx.new(|_| AccountsScreen::new(db.clone()));
        cx.subscribe(&accounts, Self::on_screen_event).detach();
        let categories = cx.new(|_| CategoriesScreen::new(db.clone()));
        cx.subscribe(&categories, Self::on_screen_event).detach();
        let subscriptions = cx.new(|cx| SubscriptionsScreen::new(db.clone(), window, cx));
        cx.subscribe(&subscriptions, Self::on_screen_event).detach();
        let projections = cx.new(|cx| ProjectionsScreen::new(db.clone(), window, cx));
        let transactions = cx.new(|cx| TransactionsScreen::new(db.clone(), window, cx));
        let text_scale = clamp_text_scale(prefs.text_scale);
        super::theme_bridge::set_text_scale(text_scale, window, cx);
        let settings = cx.new(|cx| SettingsScreen::new(text_scale, window, cx));
        cx.subscribe_in(&settings, window, |this, _, event: &SettingsEvent, window, cx| {
            let SettingsEvent::TextScale(percent) = event;
            this.set_text_scale(*percent, window, cx);
        })
        .detach();
        cx.subscribe(&transactions, Self::on_screen_event).detach();
        let overview = cx.new(|cx| OverviewScreen::new(db.clone(), window, cx));
        Self {
            overview,
            accounts,
            categories,
            subscriptions,
            projections,
            transactions,
            settings,
            toast: None,
            toast_seq: 0,
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
        match view {
            View::Overview => self.overview.update(cx, |s, cx| s.reload(window, cx)),
            View::Accounts => self.accounts.update(cx, |s, cx| s.reload(cx)),
            View::Categories => self.categories.update(cx, |s, cx| s.reload(cx)),
            View::Subscriptions => self.subscriptions.update(cx, |s, cx| s.reload(cx)),
            View::Projections => self.projections.update(cx, |s, cx| s.reload(cx)),
            View::Transactions => self.transactions.update(cx, |s, cx| s.reload(cx)),
            _ => {}
        }
        cx.notify();
    }

    fn on_screen_event<T: 'static>(&mut self, _: Entity<T>, event: &ScreenEvent, cx: &mut Context<Self>) {
        let ScreenEvent::Toast { message, ok } = event;
        self.toast_seq += 1;
        let seq = self.toast_seq;
        self.toast = Some((seq, message.clone(), *ok));
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(std::time::Duration::from_secs(3)).await;
            this.update(cx, |this, cx| {
                if this.toast.as_ref().is_some_and(|(s, ..)| *s == seq) {
                    this.toast = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    #[cfg(test)]
    pub(super) fn toast_message(&self) -> Option<&str> {
        self.toast.as_ref().map(|(_, m, _)| m.as_str())
    }

    #[cfg(test)]
    pub fn text_scale(&self) -> u16 {
        self.prefs.text_scale
    }

    /// Resize all text by scaling the window's rem, as a browser zoom would.
    pub fn set_text_scale(&mut self, percent: u16, window: &mut Window, cx: &mut Context<Self>) {
        let percent = clamp_text_scale(percent);
        self.prefs.text_scale = percent;
        super::theme_bridge::set_text_scale(percent, window, cx);
        if self.persist {
            self.prefs.save();
        }
        self.settings.update(cx, |s, cx| s.show_scale(percent, cx));
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
            Command::AddTransaction => {
                self.navigate(View::Transactions, window, cx);
                self.transactions.update(cx, |s, cx| s.open_add(window, cx));
                return;
            }
            Command::ImportTransactions => {
                self.navigate(View::Transactions, window, cx);
                self.transactions.update(cx, |s, cx| s.open_import(window, cx));
                return;
            }
            Command::AddAccount => {
                self.navigate(View::Accounts, window, cx);
                self.accounts.update(cx, |s, cx| s.open_add(window, cx));
                return;
            }
            Command::AddCategory => {
                self.navigate(View::Categories, window, cx);
                self.categories.update(cx, |s, cx| s.open_add(window, cx));
                return;
            }
            Command::AddSubscription => {
                self.navigate(View::Subscriptions, window, cx);
                self.subscriptions.update(cx, |s, cx| s.open_add(window, cx));
                return;
            }
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
            View::Overview => self.overview.clone().into_any_element(),
            View::Accounts => self.accounts.clone().into_any_element(),
            View::Categories => self.categories.clone().into_any_element(),
            View::Subscriptions => self.subscriptions.clone().into_any_element(),
            View::Projections => self.projections.clone().into_any_element(),
            View::Transactions => self.transactions.clone().into_any_element(),
            View::Settings => self.settings.clone().into_any_element(),
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
            .on_action(cx.listener(|this, _: &ZoomIn, window, cx| {
                this.set_text_scale(step_text_scale(this.prefs.text_scale, 1), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ZoomOut, window, cx| {
                this.set_text_scale(step_text_scale(this.prefs.text_scale, -1), window, cx)
            }))
            .on_action(cx.listener(|this, _: &ZoomReset, window, cx| {
                this.set_text_scale(DEFAULT_TEXT_SCALE, window, cx)
            }))
            .size_full()
            .relative()
            .flex()
            .bg(t.background)
            .text_color(t.foreground)
            .font_family(t.font.clone())
            // Everything not sized explicitly (tables, lists, labels) inherits the
            // same 12px base the Omarchy controls use.
            .text_size(rems(0.75))
            .child(sidebar(self, cx))
            .child(div().flex_1().min_w_0().h_full().child(content))
            .when_some(self.palette.clone(), |el, palette| el.child(palette))
            .when_some(self.toast.clone(), |el, (_, message, ok)| {
                el.child(
                    div()
                        .absolute()
                        .bottom(rems(1.25))
                        .right(rems(1.25))
                        .px(rems(0.875))
                        .py(rems(0.625))
                        .border_1()
                        .border_color(if ok { t.success } else { t.danger })
                        .bg(t.surface)
                        .text_color(if ok { t.success } else { t.danger })
                        .child(message),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::test_support::{init, seeded_db};
    use gpui_kit::TestAppContext;

    fn setup(cx: &mut TestAppContext) -> (Entity<Shell>, &mut gpui_kit::VisualTestContext) {
        init(cx);
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
            let overview = shell.overview.read(cx).overview().expect("overview loads");
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

    #[gpui_kit::test]
    fn palette_add_account_opens_the_form(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-k");
        cx.simulate_input("add account");
        cx.simulate_keystrokes("enter");
        assert_eq!(view_of(&shell, cx), View::Accounts);
        cx.update(|_, cx| {
            let accounts = shell.read(cx).accounts.clone();
            assert!(accounts.read(cx).has_dialog(), "add form is open");
        });
    }

    #[gpui_kit::test]
    fn toasts_show_then_clear_themselves(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        let accounts = cx.update(|_, cx| shell.read(cx).accounts.clone());
        cx.update(|_, cx| accounts.update(cx, |_, cx| cx.emit(ScreenEvent::ok("Saved it"))));
        cx.update(|_, cx| assert_eq!(shell.read(cx).toast_message(), Some("Saved it")));
        cx.executor().advance_clock(std::time::Duration::from_secs(4));
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(shell.read(cx).toast_message(), None, "gone after three seconds"));
    }

    #[gpui_kit::test]
    fn a_newer_toast_is_not_cleared_by_an_older_timer(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        let accounts = cx.update(|_, cx| shell.read(cx).accounts.clone());
        cx.update(|_, cx| accounts.update(cx, |_, cx| cx.emit(ScreenEvent::ok("first"))));
        cx.executor().advance_clock(std::time::Duration::from_secs(2));
        cx.update(|_, cx| accounts.update(cx, |_, cx| cx.emit(ScreenEvent::error("second"))));
        cx.executor().advance_clock(std::time::Duration::from_millis(1500));
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(shell.read(cx).toast_message(), Some("second"), "first timer fired but must not clear the newer toast"));
    }

    #[gpui_kit::test]
    fn every_view_renders(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        for view in View::ALL {
            cx.update(|window, cx| shell.update(cx, |s, cx| s.navigate(view, window, cx)));
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            assert_eq!(view_of(&shell, cx), view);
        }
    }

    fn rem(cx: &mut gpui_kit::VisualTestContext) -> f32 {
        cx.update(|window, _| f32::from(window.rem_size()))
    }

    #[gpui_kit::test]
    fn text_starts_at_the_default_size(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 110));
        assert!((rem(cx) - 17.6).abs() < 1e-3);
    }

    #[gpui_kit::test]
    fn a_saved_size_is_applied_on_startup(cx: &mut TestAppContext) {
        init(cx);
        let db = seeded_db();
        let prefs = Prefs { text_scale: 150, ..Prefs::default() };
        let (shell, cx) = cx.add_window_view(move |window, cx| Shell::new(db, prefs, false, window, cx));
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 150));
        assert_eq!(rem(cx), 24.0);
    }

    #[gpui_kit::test]
    fn a_nonsense_saved_size_is_pulled_into_range(cx: &mut TestAppContext) {
        init(cx);
        let db = seeded_db();
        let prefs = Prefs { text_scale: 9000, ..Prefs::default() };
        let (_shell, cx) = cx.add_window_view(move |window, cx| Shell::new(db, prefs, false, window, cx));
        assert_eq!(rem(cx), 32.0, "clamped to 200%");
    }

    #[gpui_kit::test]
    fn ctrl_plus_minus_and_zero_change_the_text_size(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        cx.simulate_keystrokes("ctrl-=");
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 125));
        assert_eq!(rem(cx), 20.0);
        cx.simulate_keystrokes("ctrl--");
        cx.simulate_keystrokes("ctrl--");
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 100));
        assert_eq!(rem(cx), 16.0);
        cx.simulate_keystrokes("ctrl-0");
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 110), );
    }

    #[gpui_kit::test]
    fn zooming_stops_at_the_smallest_and_largest_sizes(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        for _ in 0..12 {
            cx.simulate_keystrokes("ctrl--");
        }
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 70));
        for _ in 0..12 {
            cx.simulate_keystrokes("ctrl-=");
        }
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 200));
        assert_eq!(rem(cx), 32.0);
    }

    #[gpui_kit::test]
    fn the_settings_dropdown_changes_the_text_size(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        let settings = cx.update(|_, cx| shell.read(cx).settings.clone());
        cx.update(|_, cx| settings.update(cx, |s, cx| s.choose(150, cx)));
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 150));
        assert_eq!(rem(cx), 24.0);
    }

    #[gpui_kit::test]
    fn the_keyboard_updates_the_settings_dropdown_without_looping(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        let settings = cx.update(|_, cx| shell.read(cx).settings.clone());
        cx.simulate_keystrokes("ctrl-=");
        cx.update(|_, cx| {
            assert_eq!(settings.read(cx).shown(), 125, "dropdown follows the shortcut");
            assert_eq!(shell.read(cx).text_scale(), 125, "and does not push a second change back");
        });
        cx.simulate_keystrokes("ctrl-0");
        cx.update(|_, cx| assert_eq!(settings.read(cx).shown(), 110));
    }

    fn measure(cx: &mut gpui_kit::VisualTestContext, selector: &'static str) -> f32 {
        cx.update(|window, cx| window.draw(cx).clear(cx));
        f32::from(cx.debug_bounds(selector).unwrap_or_else(|| panic!("{selector} not drawn")).size.width)
    }

    #[gpui_kit::test]
    fn zooming_actually_resizes_what_is_drawn(cx: &mut TestAppContext) {
        // Navigate somewhere with a page title first.
        let (shell, cx) = setup(cx);
        cx.update(|window, cx| shell.update(cx, |s, cx| s.navigate(View::Accounts, window, cx)));
        let (sidebar, title) = (measure(cx, "sidebar"), measure(cx, "page-title"));

        cx.simulate_keystrokes("ctrl-=");
        let (sidebar_big, title_big) = (measure(cx, "sidebar"), measure(cx, "page-title"));
        // 110% -> 125% is a 13.6% increase.
        assert!(sidebar_big > sidebar * 1.1, "sidebar should widen: {sidebar} -> {sidebar_big}");
        assert!(title_big > title * 1.1, "title text should grow: {title} -> {title_big}");

        cx.simulate_keystrokes("ctrl-0");
        let (sidebar_back, title_back) = (measure(cx, "sidebar"), measure(cx, "page-title"));
        assert!((sidebar_back - sidebar).abs() < 0.5, "reset restores the width");
        assert!((title_back - title).abs() < 0.5, "reset restores the text");
    }

    #[gpui_kit::test]
    fn the_settings_dropdown_resizes_the_layout_too(cx: &mut TestAppContext) {
        let (shell, cx) = setup(cx);
        let before = measure(cx, "sidebar");
        let settings = cx.update(|_, cx| shell.read(cx).settings.clone());
        cx.update(|_, cx| settings.update(cx, |s, cx| s.choose(200, cx)));
        let after = measure(cx, "sidebar");
        assert!(after > before * 1.5, "200% vs 110% should nearly double the sidebar: {before} -> {after}");
    }

    /// The real app opens its window through `gpui_kit::open_window`, which
    /// wraps the shell in a Root view. Tests normally skip that wrapper.
    #[gpui_kit::test]
    fn zooming_works_in_a_window_opened_the_way_main_opens_it(cx: &mut TestAppContext) {
        init(cx);
        let db = seeded_db();
        let (window, shell) = cx
            .update(|cx| {
                gpui_kit::open_window(gpui_kit::WindowOptions::default(), cx, |window, cx| {
                    cx.new(|cx| Shell::new(db, Prefs::default(), false, window, cx))
                })
            })
            .expect("window opens");
        let cx = &mut gpui_kit::VisualTestContext::from_window(window, cx);
        let sidebar_is = |cx: &mut gpui_kit::VisualTestContext, expected: f32| {
            // Draw a few frames: Root re-applies its own rem size on every one.
            for _ in 0..3 {
                cx.update(|window, cx| window.draw(cx).clear(cx));
            }
            let width = measure(cx, "sidebar");
            assert!((width - expected).abs() < 0.6, "sidebar is 15rem = {expected}px, was {width}px");
        };

        sidebar_is(cx, 15.0 * 17.6); // 110% is the default
        cx.simulate_keystrokes("ctrl-=");
        cx.update(|_, cx| assert_eq!(shell.read(cx).text_scale(), 125));
        sidebar_is(cx, 15.0 * 20.0);
        cx.simulate_keystrokes("ctrl--");
        cx.simulate_keystrokes("ctrl--");
        sidebar_is(cx, 15.0 * 16.0);
        cx.simulate_keystrokes("ctrl-0");
        sidebar_is(cx, 15.0 * 17.6);

        let settings = cx.update(|_, cx| shell.read(cx).settings.clone());
        cx.update(|_, cx| settings.update(cx, |s, cx| s.choose(200, cx)));
        sidebar_is(cx, 15.0 * 32.0);
    }
}
