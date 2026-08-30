use rusqlite::{Connection, Result};
use std::fs;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

pub struct DbState(pub Mutex<Connection>);

pub fn init_database(app: &AppHandle) -> Result<Mutex<Connection>> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("failed to create app data dir");

    let db_path = app_dir.join("chelete.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    run_migrations(&conn)?;

    Ok(Mutex::new(conn))
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            account_type TEXT NOT NULL DEFAULT 'cash',
            currency TEXT NOT NULL DEFAULT 'USD',
            balance INTEGER NOT NULL DEFAULT 0,
            color TEXT,
            icon TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT
        );

        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id TEXT,
            category_type TEXT NOT NULL DEFAULT 'expense',
            icon TEXT,
            color TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT,
            FOREIGN KEY (parent_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            category_id TEXT,
            transaction_type TEXT NOT NULL DEFAULT 'expense',
            amount INTEGER NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            description TEXT NOT NULL,
            merchant TEXT,
            notes TEXT,
            transaction_date TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT,
            FOREIGN KEY (account_id) REFERENCES accounts(id),
            FOREIGN KEY (category_id) REFERENCES categories(id)
        );

        CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(transaction_date);
        CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
        CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
        CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
        ",
    )?;

    Ok(())
}
