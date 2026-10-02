use super::accounts::AccountsScreen;
use super::categories::CategoriesScreen;
use super::projections::ProjectionsScreen;
use super::settings;
use super::subscriptions::SubscriptionsScreen;
use super::overview::{self, OverviewData};
use super::widgets::ScreenEvent;
use super::palette::{Command, Palette, PaletteEvent};
use super::placeholder::placeholder;
use super::sidebar::sidebar;
use super::*;
use chelete_lib::database::DbState;
use chelete_lib::prefs::Prefs;
use gpui_kit::{
    div, prelude::*, rems, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Subscription,
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
    accounts: Entity<AccountsScreen>,
    categories: Entity<CategoriesScreen>,
    subscriptions: Entity<SubscriptionsScreen>,
    projections: Entity<ProjectionsScreen>,
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
        Self {
            overview: OverviewData::load(&db),
            accounts,
            categories,
            subscriptions,
            projections,
            toast: None,
            toast_seq: 0,
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
        match view {
            View::Overview => self.overview = OverviewData::load(&self.db),
            View::Accounts => self.accounts.update(cx, |s, cx| s.reload(cx)),
            View::Categories => self.categories.update(cx, |s, cx| s.reload(cx)),
            View::Subscriptions => self.subscriptions.update(cx, |s, cx| s.reload(cx)),
            View::Projections => self.projections.update(cx, |s, cx| s.reload(cx)),
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
            View::Overview => overview::render(&self.overview, cx).into_any_element(),
            View::Accounts => self.accounts.clone().into_any_element(),
            View::Categories => self.categories.clone().into_any_element(),
            View::Subscriptions => self.subscriptions.clone().into_any_element(),
            View::Projections => self.projections.clone().into_any_element(),
            View::Settings => settings::render(cx).into_any_element(),
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
}
