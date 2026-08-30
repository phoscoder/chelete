use crate::database::DbState;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

fn uid() -> String {
    Uuid::new_v4().to_string()
}

pub fn seed_database(conn: &rusqlite::Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Ok(false);
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let last_month = (chrono::Utc::now() - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    // ── Accounts ──────────────────────────────────────────────────
    let checking_id = uid();
    let savings_id = uid();
    let cash_id = uid();
    let credit_id = uid();

    let accounts = vec![
        (checking_id.clone(), "Checking Account", "bank", "USD", 5250_00i64, "#4a9eff", "building-2"),
        (savings_id.clone(), "Savings", "savings", "USD", 15000_00i64, "#22c55e", "piggy-bank"),
        (cash_id.clone(), "Cash Wallet", "cash", "USD", 850_00i64, "#f59e0b", "wallet"),
        (credit_id.clone(), "Credit Card", "credit_card", "USD", -1200_00i64, "#ef4444", "credit-card"),
    ];

    for (i, (id, name, acc_type, currency, balance, color, icon)) in accounts.iter().enumerate() {
        conn.execute(
            "INSERT INTO accounts (id, name, account_type, currency, balance, color, icon, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, name, acc_type, currency, balance, color, icon, i as i32, now, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // ── Categories ────────────────────────────────────────────────
    let cat_food = uid();
    let cat_transport = uid();
    let cat_salary = uid();
    let cat_entertainment = uid();
    let cat_shopping = uid();
    let cat_bills = uid();
    let cat_health = uid();
    let cat_education = uid();

    let categories = vec![
        (cat_food.clone(), "Food & Dining", "expense", "utensils", "#f97316", 0),
        (cat_transport.clone(), "Transportation", "expense", "car", "#3b82f6", 1),
        (cat_salary.clone(), "Salary", "income", "briefcase", "#22c55e", 2),
        (cat_entertainment.clone(), "Entertainment", "expense", "film", "#a855f7", 3),
        (cat_shopping.clone(), "Shopping", "expense", "shopping-bag", "#ec4899", 4),
        (cat_bills.clone(), "Bills & Utilities", "expense", "zap", "#eab308", 5),
        (cat_health.clone(), "Health", "expense", "heart", "#ef4444", 6),
        (cat_education.clone(), "Education", "expense", "book-open", "#06b6d4", 7),
    ];

    for (id, name, cat_type, icon, color, sort_order) in &categories {
        conn.execute(
            "INSERT INTO categories (id, name, category_type, icon, color, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, cat_type, icon, color, sort_order, now, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // ── Transactions ──────────────────────────────────────────────
    let transactions: Vec<(
        &str, &str, &str, i64, &str, &str, Option<&str>, Option<&str>, &str,
    )> = vec![
        // Income
        ("income", &checking_id, &cat_salary, 4500_00, "Monthly Salary", "Acme Corp", Some("March salary"), None, &today),
        ("income", &checking_id, &cat_salary, 4500_00, "Monthly Salary", "Acme Corp", Some("February salary"), None, &last_month),
        ("income", &savings_id, &cat_salary, 1500_00, "Freelance Payment", "Client X", Some("Web project"), None, &today),

        // Expenses - Checking
        ("expense", &checking_id, &cat_food, 45_50, "Grocery run", "Whole Foods", None, None, &today),
        ("expense", &checking_id, &cat_food, 28_00, "Dinner out", "Italian Bistro", Some("Date night"), None, &today),
        ("expense", &checking_id, &cat_food, 12_50, "Coffee & pastry", "Blue Bottle", None, None, &today),
        ("expense", &checking_id, &cat_transport, 65_00, "Gas fill-up", "Shell", None, None, &today),
        ("expense", &checking_id, &cat_transport, 35_00, "Uber ride", "Uber", Some("Airport"), None, &last_month),
        ("expense", &checking_id, &cat_entertainment, 15_99, "Netflix subscription", "Netflix", None, None, &today),
        ("expense", &checking_id, &cat_bills, 120_00, "Electric bill", "City Power", None, None, &today),
        ("expense", &checking_id, &cat_bills, 85_00, "Internet bill", "Comcast", None, None, &last_month),
        ("expense", &checking_id, &cat_health, 30_00, "Pharmacy", "CVS", None, None, &today),
        ("expense", &checking_id, &cat_education, 49_99, "Online course", "Udemy", Some("Rust programming"), None, &last_month),

        // Expenses - Credit Card
        ("expense", &credit_id, &cat_shopping, 199_99, "New headphones", "Amazon", Some("Noise cancelling"), None, &today),
        ("expense", &credit_id, &cat_shopping, 45_00, "T-shirt", "Uniqlo", None, None, &last_month),
        ("expense", &credit_id, &cat_food, 52_00, "Sushi dinner", "Sushi Zen", None, None, &last_month),
        ("expense", &credit_id, &cat_entertainment, 12_00, "Movie tickets", "AMC", Some("2 tickets"), None, &last_month),

        // Expenses - Cash
        ("expense", &cash_id, &cat_food, 15_00, "Lunch", "Food truck", None, None, &today),
        ("expense", &cash_id, &cat_transport, 20_00, "Parking", "Downtown garage", None, None, &today),

        // Last month expenses
        ("expense", &checking_id, &cat_food, 62_30, "Weekly groceries", "Trader Joe's", None, None, &last_month),
        ("expense", &checking_id, &cat_food, 38_00, "Brunch", "The Original Pancake House", None, None, &last_month),
        ("expense", &checking_id, &cat_transport, 150_00, "Car maintenance", "Jiffy Lube", Some("Oil change"), None, &last_month),
        ("expense", &checking_id, &cat_bills, 200_00, "Rent supplement", "Landlord", None, None, &last_month),
        ("expense", &credit_id, &cat_shopping, 89_99, "Running shoes", "Nike", None, None, &last_month),
        ("expense", &cash_id, &cat_food, 22_00, "Street food", "Night market", None, None, &last_month),
    ];

    for (txn_type, account_id, category_id, amount, desc, merchant, notes, _unused, txn_date) in &transactions {
        let txn_id = uid();
        conn.execute(
            "INSERT INTO transactions (id, account_id, category_id, transaction_type, amount, currency, description, merchant, notes, transaction_date, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'USD', ?6, ?7, ?8, ?9, ?10, ?11)",
            params![txn_id, account_id, category_id, txn_type, amount, desc, merchant, notes, txn_date, now, now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(true)
}

#[tauri::command]
pub fn seed_database_command(state: State<'_, DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    seed_database(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        crate::database::run_migrations_for_test(&conn).unwrap();
        conn
    }

    #[test]
    fn test_seed_populates_data() {
        let conn = test_conn();
        let result = seed_database(&conn).unwrap();
        assert!(result, "seed should return true on first run");

        let accounts: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(accounts, 4);

        let categories: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(categories, 8);

        let transactions: i64 = conn
            .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(transactions, 25);
    }

    #[test]
    fn test_seed_is_idempotent() {
        let conn = test_conn();
        seed_database(&conn).unwrap();
        let result = seed_database(&conn).unwrap();
        assert!(!result, "seed should return false on second run");

        let accounts: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(accounts, 4, "should still have exactly 4 accounts");
    }
}
