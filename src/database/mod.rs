use rusqlite::{Connection, Result};
use std::fs;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);

/// Same directory the Tauri build used, so existing data keeps loading.
pub fn data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .expect("failed to get data dir")
        .join("com.chelete.app")
}

pub fn init_database() -> Result<Mutex<Connection>> {
    let app_dir = data_dir();
    fs::create_dir_all(&app_dir).expect("failed to create app data dir");

    let db_path = app_dir.join("chelete.db");
    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    run_migrations(&conn)?;

    Ok(Mutex::new(conn))
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
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

        CREATE TABLE IF NOT EXISTS subscriptions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            amount INTEGER NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            frequency TEXT NOT NULL DEFAULT 'monthly',
            category_id TEXT,
            account_id TEXT,
            start_date TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            deleted_at TEXT,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (account_id) REFERENCES accounts(id)
        );

        CREATE INDEX IF NOT EXISTS idx_subscriptions_category ON subscriptions(category_id);
        CREATE INDEX IF NOT EXISTS idx_subscriptions_account ON subscriptions(account_id);
        CREATE INDEX IF NOT EXISTS idx_subscriptions_frequency ON subscriptions(frequency);
        ",
    )?;

    // Accounts deleted before deletes cascaded left their transactions behind.
    conn.execute_batch(
        "
        UPDATE transactions SET deleted_at = datetime('now')
        WHERE deleted_at IS NULL
          AND account_id IN (SELECT id FROM accounts WHERE deleted_at IS NOT NULL);

        UPDATE subscriptions SET account_id = NULL
        WHERE account_id IN (SELECT id FROM accounts WHERE deleted_at IS NOT NULL);
        ",
    )?;

    Ok(())
}

#[cfg(test)]
pub fn run_migrations_for_test(conn: &Connection) -> Result<()> {
    run_migrations(conn)
}
