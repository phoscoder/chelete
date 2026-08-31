use crate::database::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub merchant: Option<String>,
    pub notes: Option<String>,
    pub transaction_date: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Overview {
    pub total_balance: i64,
    pub total_income: i64,
    pub total_expenses: i64,
    pub accounts: Vec<Account>,
    pub recent_transactions: Vec<Transaction>,
    pub category_spending: Vec<CategorySpending>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategorySpending {
    pub category_id: String,
    pub category_name: String,
    pub spent: i64,
    pub budget_limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub amount: i64,
    pub currency: String,
    pub frequency: String,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
    pub start_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub account_type: String,
    pub currency: String,
    pub balance: i64,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: String,
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub currency: Option<String>,
    pub balance: Option<i64>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: String,
    pub category_id: Option<String>,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub merchant: Option<String>,
    pub notes: Option<String>,
    pub transaction_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTransactionRequest {
    pub id: String,
    pub account_id: Option<String>,
    pub category_id: Option<Option<String>>,
    pub transaction_type: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub merchant: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub transaction_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub parent_id: Option<String>,
    pub category_type: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCategoryRequest {
    pub id: String,
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub category_type: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub amount: i64,
    pub currency: String,
    pub frequency: String,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
    pub start_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub id: String,
    pub name: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub frequency: Option<String>,
    pub category_id: Option<Option<String>>,
    pub account_id: Option<Option<String>>,
    pub start_date: Option<Option<String>>,
    pub is_active: Option<bool>,
}

fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ── Accounts ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_accounts(state: State<'_, DbState>) -> Result<Vec<Account>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, account_type, currency, balance, color, icon, is_active, sort_order, created_at, updated_at
             FROM accounts WHERE deleted_at IS NULL ORDER BY sort_order",
        )
        .map_err(|e| e.to_string())?;

    let accounts = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                currency: row.get(3)?,
                balance: row.get(4)?,
                color: row.get(5)?,
                icon: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(accounts)
}

#[tauri::command]
pub fn create_account(
    state: State<'_, DbState>,
    request: CreateAccountRequest,
) -> Result<Account, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = generate_id();

    conn.execute(
        "INSERT INTO accounts (id, name, account_type, currency, balance, color, icon)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            request.name,
            request.account_type,
            request.currency,
            request.balance,
            request.color,
            request.icon,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Account {
        id,
        name: request.name,
        account_type: request.account_type,
        currency: request.currency,
        balance: request.balance,
        color: request.color,
        icon: request.icon,
        is_active: true,
        sort_order: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn update_account(
    state: State<'_, DbState>,
    request: UpdateAccountRequest,
) -> Result<Account, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(name) = &request.name {
        conn.execute(
            "UPDATE accounts SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(account_type) = &request.account_type {
        conn.execute(
            "UPDATE accounts SET account_type = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![account_type, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(balance) = request.balance {
        conn.execute(
            "UPDATE accounts SET balance = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![balance, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(is_active) = request.is_active {
        conn.execute(
            "UPDATE accounts SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![is_active as i32, request.id],
        )
        .map_err(|e| e.to_string())?;
    }

    let account = get_account_by_id(&conn, &request.id)?;
    Ok(account)
}

#[tauri::command]
pub fn delete_account(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE accounts SET deleted_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_account_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Account, String> {
    conn.query_row(
        "SELECT id, name, account_type, currency, balance, color, icon, is_active, sort_order, created_at, updated_at
         FROM accounts WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                currency: row.get(3)?,
                balance: row.get(4)?,
                color: row.get(5)?,
                icon: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

// ── Transactions ─────────────────────────────────────────────────

#[tauri::command]
pub fn get_transactions(state: State<'_, DbState>) -> Result<Vec<Transaction>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, account_id, category_id, transaction_type, amount, currency, description, merchant, notes, transaction_date, created_at, updated_at
             FROM transactions WHERE deleted_at IS NULL ORDER BY transaction_date DESC, created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let transactions = stmt
        .query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category_id: row.get(2)?,
                transaction_type: row.get(3)?,
                amount: row.get(4)?,
                currency: row.get(5)?,
                description: row.get(6)?,
                merchant: row.get(7)?,
                notes: row.get(8)?,
                transaction_date: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(transactions)
}

#[tauri::command]
pub fn create_transaction(
    state: State<'_, DbState>,
    request: CreateTransactionRequest,
) -> Result<Transaction, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = generate_id();

    conn.execute(
        "INSERT INTO transactions (id, account_id, category_id, transaction_type, amount, currency, description, merchant, notes, transaction_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            request.account_id,
            request.category_id,
            request.transaction_type,
            request.amount,
            request.currency,
            request.description,
            request.merchant,
            request.notes,
            request.transaction_date,
        ],
    )
    .map_err(|e| e.to_string())?;

    // Update account balance
    let balance_change = if request.transaction_type == "income" {
        request.amount
    } else {
        -request.amount
    };
    conn.execute(
        "UPDATE accounts SET balance = balance + ?1, updated_at = datetime('now') WHERE id = ?2",
        params![balance_change, request.account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Transaction {
        id,
        account_id: request.account_id,
        category_id: request.category_id,
        transaction_type: request.transaction_type,
        amount: request.amount,
        currency: request.currency,
        description: request.description,
        merchant: request.merchant,
        notes: request.notes,
        transaction_date: request.transaction_date,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn update_transaction(
    state: State<'_, DbState>,
    request: UpdateTransactionRequest,
) -> Result<Transaction, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Get old transaction for balance adjustment
    let old = get_transaction_by_id(&conn, &request.id)?;

    if let Some(amount) = request.amount {
        if amount != old.amount || request.transaction_type.as_deref() != Some(&old.transaction_type) {
            // Reverse old balance change
            let old_change = if old.transaction_type == "income" {
                -old.amount
            } else {
                old.amount
            };
            conn.execute(
                "UPDATE accounts SET balance = balance + ?1, updated_at = datetime('now') WHERE id = ?2",
                params![old_change, old.account_id],
            )
            .map_err(|e| e.to_string())?;

            // Apply new balance change
            let new_type = request.transaction_type.as_deref().unwrap_or(old.transaction_type.as_str());
            let new_change = if new_type == "income" { amount } else { -amount };
            let account_id = request.account_id.as_deref().unwrap_or(&old.account_id);
            conn.execute(
                "UPDATE accounts SET balance = balance + ?1, updated_at = datetime('now') WHERE id = ?2",
                params![new_change, account_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Update fields
    if let Some(account_id) = &request.account_id {
        conn.execute(
            "UPDATE transactions SET account_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![account_id, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(category_id) = &request.category_id {
        conn.execute(
            "UPDATE transactions SET category_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![category_id, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(transaction_type) = &request.transaction_type {
        conn.execute(
            "UPDATE transactions SET transaction_type = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![transaction_type, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(amount) = request.amount {
        conn.execute(
            "UPDATE transactions SET amount = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![amount, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(description) = &request.description {
        conn.execute(
            "UPDATE transactions SET description = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![description, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(merchant) = &request.merchant {
        conn.execute(
            "UPDATE transactions SET merchant = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![merchant, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(notes) = &request.notes {
        conn.execute(
            "UPDATE transactions SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![notes, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(transaction_date) = &request.transaction_date {
        conn.execute(
            "UPDATE transactions SET transaction_date = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![transaction_date, request.id],
        )
        .map_err(|e| e.to_string())?;
    }

    get_transaction_by_id(&conn, &request.id)
}

#[tauri::command]
pub fn delete_transaction(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let txn = get_transaction_by_id(&conn, &id)?;

    // Reverse balance change
    let change = if txn.transaction_type == "income" {
        -txn.amount
    } else {
        txn.amount
    };
    conn.execute(
        "UPDATE accounts SET balance = balance + ?1, updated_at = datetime('now') WHERE id = ?2",
        params![change, txn.account_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE transactions SET deleted_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_transaction_by_id(
    conn: &rusqlite::Connection,
    id: &str,
) -> Result<Transaction, String> {
    conn.query_row(
        "SELECT id, account_id, category_id, transaction_type, amount, currency, description, merchant, notes, transaction_date, created_at, updated_at
         FROM transactions WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| {
            Ok(Transaction {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category_id: row.get(2)?,
                transaction_type: row.get(3)?,
                amount: row.get(4)?,
                currency: row.get(5)?,
                description: row.get(6)?,
                merchant: row.get(7)?,
                notes: row.get(8)?,
                transaction_date: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

// ── Categories ───────────────────────────────────────────────────

#[tauri::command]
pub fn get_categories(state: State<'_, DbState>) -> Result<Vec<Category>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, category_type, icon, color, sort_order, created_at, updated_at
             FROM categories WHERE deleted_at IS NULL ORDER BY sort_order",
        )
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                category_type: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(categories)
}

#[tauri::command]
pub fn create_category(
    state: State<'_, DbState>,
    request: CreateCategoryRequest,
) -> Result<Category, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = generate_id();

    conn.execute(
        "INSERT INTO categories (id, name, parent_id, category_type, icon, color)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            request.name,
            request.parent_id,
            request.category_type,
            request.icon,
            request.color,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Category {
        id,
        name: request.name,
        parent_id: request.parent_id,
        category_type: request.category_type,
        icon: request.icon,
        color: request.color,
        sort_order: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn update_category(
    state: State<'_, DbState>,
    request: UpdateCategoryRequest,
) -> Result<Category, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(name) = &request.name {
        conn.execute(
            "UPDATE categories SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(category_type) = &request.category_type {
        conn.execute(
            "UPDATE categories SET category_type = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![category_type, request.id],
        )
        .map_err(|e| e.to_string())?;
    }

    let category = get_category_by_id(&conn, &request.id)?;
    Ok(category)
}

#[tauri::command]
pub fn delete_category(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE categories SET deleted_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_category_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Category, String> {
    conn.query_row(
        "SELECT id, name, parent_id, category_type, icon, color, sort_order, created_at, updated_at
         FROM categories WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                category_type: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                sort_order: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

// ── Overview ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_overview(state: State<'_, DbState>) -> Result<Overview, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Total balance from all active accounts
    let total_balance: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(balance), 0) FROM accounts WHERE deleted_at IS NULL AND is_active = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Current month income/expenses
    let current_month = chrono::Utc::now().format("%Y-%m").to_string();
    let total_income: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE deleted_at IS NULL AND transaction_type = 'income' AND transaction_date LIKE ?1",
            params![format!("{}%", current_month)],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let total_expenses: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE deleted_at IS NULL AND transaction_type = 'expense' AND transaction_date LIKE ?1",
            params![format!("{}%", current_month)],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Accounts
    let mut stmt = conn
        .prepare(
            "SELECT id, name, account_type, currency, balance, color, icon, is_active, sort_order, created_at, updated_at
             FROM accounts WHERE deleted_at IS NULL AND is_active = 1 ORDER BY sort_order",
        )
        .map_err(|e| e.to_string())?;
    let accounts = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: row.get(2)?,
                currency: row.get(3)?,
                balance: row.get(4)?,
                color: row.get(5)?,
                icon: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    // Recent transactions
    let mut stmt = conn
        .prepare(
            "SELECT id, account_id, category_id, transaction_type, amount, currency, description, merchant, notes, transaction_date, created_at, updated_at
             FROM transactions WHERE deleted_at IS NULL ORDER BY transaction_date DESC, created_at DESC LIMIT 10",
        )
        .map_err(|e| e.to_string())?;
    let recent_transactions = stmt
        .query_map([], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category_id: row.get(2)?,
                transaction_type: row.get(3)?,
                amount: row.get(4)?,
                currency: row.get(5)?,
                description: row.get(6)?,
                merchant: row.get(7)?,
                notes: row.get(8)?,
                transaction_date: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    // Category spending for current month
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, COALESCE(SUM(t.amount), 0)
             FROM categories c
             LEFT JOIN transactions t ON t.category_id = c.id
                 AND t.deleted_at IS NULL
                 AND t.transaction_type = 'expense'
                 AND t.transaction_date LIKE ?1
             WHERE c.deleted_at IS NULL AND c.category_type = 'expense'
             GROUP BY c.id
             ORDER BY SUM(t.amount) DESC",
        )
        .map_err(|e| e.to_string())?;
    let category_spending = stmt
        .query_map(params![format!("{}%", current_month)], |row| {
            Ok(CategorySpending {
                category_id: row.get(0)?,
                category_name: row.get(1)?,
                spent: row.get(2)?,
                budget_limit: None,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    Ok(Overview {
        total_balance,
        total_income,
        total_expenses,
        accounts,
        recent_transactions,
        category_spending,
    })
}

// ── Subscriptions ────────────────────────────────────────────────

#[tauri::command]
pub fn get_subscriptions(state: State<'_, DbState>) -> Result<Vec<Subscription>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, amount, currency, frequency, category_id, account_id, start_date, is_active, created_at, updated_at
             FROM subscriptions WHERE deleted_at IS NULL ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let subscriptions = stmt
        .query_map([], |row| {
            Ok(Subscription {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                currency: row.get(3)?,
                frequency: row.get(4)?,
                category_id: row.get(5)?,
                account_id: row.get(6)?,
                start_date: row.get(7)?,
                is_active: row.get::<_, i32>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(subscriptions)
}

#[tauri::command]
pub fn create_subscription(
    state: State<'_, DbState>,
    request: CreateSubscriptionRequest,
) -> Result<Subscription, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = generate_id();

    conn.execute(
        "INSERT INTO subscriptions (id, name, amount, currency, frequency, category_id, account_id, start_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            request.name,
            request.amount,
            request.currency,
            request.frequency,
            request.category_id,
            request.account_id,
            request.start_date,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Subscription {
        id,
        name: request.name,
        amount: request.amount,
        currency: request.currency,
        frequency: request.frequency,
        category_id: request.category_id,
        account_id: request.account_id,
        start_date: request.start_date,
        is_active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub fn update_subscription(
    state: State<'_, DbState>,
    request: UpdateSubscriptionRequest,
) -> Result<Subscription, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(name) = &request.name {
        conn.execute(
            "UPDATE subscriptions SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![name, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(amount) = request.amount {
        conn.execute(
            "UPDATE subscriptions SET amount = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![amount, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(currency) = &request.currency {
        conn.execute(
            "UPDATE subscriptions SET currency = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![currency, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(frequency) = &request.frequency {
        conn.execute(
            "UPDATE subscriptions SET frequency = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![frequency, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(category_id) = &request.category_id {
        conn.execute(
            "UPDATE subscriptions SET category_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![category_id, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(account_id) = &request.account_id {
        conn.execute(
            "UPDATE subscriptions SET account_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![account_id, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(start_date) = &request.start_date {
        conn.execute(
            "UPDATE subscriptions SET start_date = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![start_date, request.id],
        )
        .map_err(|e| e.to_string())?;
    }
    if let Some(is_active) = request.is_active {
        let active = if is_active { 1 } else { 0 };
        conn.execute(
            "UPDATE subscriptions SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![active, request.id],
        )
        .map_err(|e| e.to_string())?;
    }

    get_subscription_by_id(&conn, &request.id)
}

#[tauri::command]
pub fn delete_subscription(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE subscriptions SET deleted_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_subscription_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Subscription, String> {
    conn.query_row(
        "SELECT id, name, amount, currency, frequency, category_id, account_id, start_date, is_active, created_at, updated_at
         FROM subscriptions WHERE id = ?1 AND deleted_at IS NULL",
        params![id],
        |row| {
            Ok(Subscription {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                currency: row.get(3)?,
                frequency: row.get(4)?,
                category_id: row.get(5)?,
                account_id: row.get(6)?,
                start_date: row.get(7)?,
                is_active: row.get::<_, i32>(8)? == 1,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

// ── Theme ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_omarchy_theme() -> Result<crate::theme::OmarchyTheme, String> {
    crate::theme::detect_theme()
}
