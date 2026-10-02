mod ui;

use chelete_lib::database::{self, DbState};
use chelete_lib::prefs::Prefs;
use gpui_kit::{px, size, AppContext as _, Bounds, WindowBounds, WindowOptions};
use std::sync::Arc;

fn main() {
    let db = Arc::new(DbState(
        database::init_database().expect("failed to open database"),
    ));

    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);
            gpui_omarchy::init(cx);
            ui::bind_keys(cx);
            cx.on_action(|_: &ui::Quit, cx| cx.quit());
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1100.), px(700.)),
                    cx,
                ))),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| ui::Shell::new(db, Prefs::load(), true, window, cx))
            })
            .expect("open window");
            cx.activate(true);
        });
}
