//! Shared helpers for headless UI tests.
use chelete_lib::database::{self, DbState};
use chelete_lib::seed;
use gpui_kit::TestAppContext;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub fn seeded_db() -> Arc<DbState> {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    database::run_migrations(&conn).unwrap();
    seed::seed_database(&conn).unwrap();
    Arc::new(DbState(Mutex::new(conn)))
}

pub fn empty_db() -> Arc<DbState> {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    database::run_migrations(&conn).unwrap();
    Arc::new(DbState(Mutex::new(conn)))
}

pub fn init(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        gpui_omarchy::init(cx);
        // Applying a fixed theme stops the filesystem watcher, which the
        // deterministic test scheduler rejects.
        gpui_omarchy::Theme::tokyo_night().apply(cx);
        super::install(cx);
    });
}
