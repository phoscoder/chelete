//! Dashboard figures for a chosen date range.
use crate::commands::{Category, Transaction};
use crate::date_filter::is_date_in_range;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTotal {
    pub category_id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub total_income: i64,
    pub total_expenses: i64,
    pub net: i64,
    /// Whole percent of income kept; zero without income.
    pub savings_rate: i64,
    pub income_by_category: Vec<CategoryTotal>,
    pub expenses_by_category: Vec<CategoryTotal>,
}

fn totals(map: HashMap<String, i64>, categories: &[Category]) -> Vec<CategoryTotal> {
    let mut list: Vec<CategoryTotal> = map
        .into_iter()
        .map(|(category_id, value)| {
            let category = categories.iter().find(|c| c.id == category_id);
            CategoryTotal {
                name: category.map_or_else(|| "Unknown".to_string(), |c| c.name.clone()),
                color: category.and_then(|c| c.color.clone()),
                icon: category.and_then(|c| c.icon.clone()),
                category_id,
                value,
            }
        })
        .collect();
    // Largest first; ties by name so the order is stable.
    list.sort_by(|a, b| b.value.cmp(&a.value).then_with(|| a.name.cmp(&b.name)));
    list
}

/// Totals for transactions dated within `start..=end`.
pub fn summarize(transactions: &[Transaction], categories: &[Category], start: &str, end: &str) -> Summary {
    let (mut income, mut expenses) = (0i64, 0i64);
    let mut income_by: HashMap<String, i64> = HashMap::new();
    let mut expense_by: HashMap<String, i64> = HashMap::new();
    for t in transactions.iter().filter(|t| is_date_in_range(&t.transaction_date, start, end)) {
        match t.transaction_type.as_str() {
            "income" => {
                income += t.amount;
                if let Some(id) = &t.category_id {
                    *income_by.entry(id.clone()).or_insert(0) += t.amount;
                }
            }
            "expense" => {
                expenses += t.amount;
                if let Some(id) = &t.category_id {
                    *expense_by.entry(id.clone()).or_insert(0) += t.amount;
                }
            }
            _ => {}
        }
    }
    let net = income - expenses;
    Summary {
        total_income: income,
        total_expenses: expenses,
        net,
        // Round half up like the web UI (Math.round).
        savings_rate: if income > 0 { (net as f64 / income as f64 * 100.0 + 0.5).floor() as i64 } else { 0 },
        income_by_category: totals(income_by, categories),
        expenses_by_category: totals(expense_by, categories),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeFilter {
    All,
    Income,
    Expense,
}

impl TypeFilter {
    pub fn from_value(value: &str) -> Self {
        match value {
            "income" => TypeFilter::Income,
            "expense" => TypeFilter::Expense,
            _ => TypeFilter::All,
        }
    }
}

/// Transactions in the date range, optionally narrowed by type and account.
pub fn filter_transactions<'a>(
    transactions: &'a [Transaction],
    start: &str,
    end: &str,
    kind: TypeFilter,
    account_id: Option<&str>,
) -> Vec<&'a Transaction> {
    transactions
        .iter()
        .filter(|t| is_date_in_range(&t.transaction_date, start, end))
        .filter(|t| match kind {
            TypeFilter::All => true,
            TypeFilter::Income => t.transaction_type == "income",
            TypeFilter::Expense => t.transaction_type == "expense",
        })
        .filter(|t| account_id.is_none_or(|id| t.account_id == id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(id: &str, kind: &str, amount: i64, date: &str, cat: Option<&str>, account: &str) -> Transaction {
        Transaction {
            id: id.into(),
            account_id: account.into(),
            category_id: cat.map(String::from),
            transaction_type: kind.into(),
            amount,
            currency: "USD".into(),
            description: id.into(),
            merchant: None,
            notes: None,
            transaction_date: date.into(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn cat(id: &str, name: &str, color: Option<&str>) -> Category {
        Category {
            id: id.into(),
            name: name.into(),
            parent_id: None,
            category_type: "expense".into(),
            icon: Some("tag".into()),
            color: color.map(String::from),
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn sample() -> Vec<Transaction> {
        vec![
            tx("pay", "income", 200000, "2026-03-01", Some("job"), "a"),
            tx("rent", "expense", 80000, "2026-03-02", Some("home"), "a"),
            tx("food1", "expense", 10000, "2026-03-03", Some("food"), "b"),
            tx("food2", "expense", 5000, "2026-03-04", Some("food"), "a"),
            tx("old", "expense", 99999, "2025-01-01", Some("food"), "a"),
            tx("loose", "expense", 700, "2026-03-05", None, "a"),
        ]
    }

    fn cats() -> Vec<Category> {
        vec![cat("job", "Salary", Some("#9ece6a")), cat("home", "Rent", None), cat("food", "Food", None)]
    }

    #[test]
    fn totals_only_include_the_date_range() {
        let s = summarize(&sample(), &cats(), "2026-03-01", "2026-03-31");
        assert_eq!(s.total_income, 200000);
        assert_eq!(s.total_expenses, 80000 + 10000 + 5000 + 700);
        assert_eq!(s.net, 200000 - 95700);
    }

    #[test]
    fn categories_are_grouped_and_sorted_largest_first() {
        let s = summarize(&sample(), &cats(), "2026-03-01", "2026-03-31");
        let names: Vec<_> = s.expenses_by_category.iter().map(|c| (c.name.as_str(), c.value)).collect();
        assert_eq!(names, vec![("Rent", 80000), ("Food", 15000)], "uncategorised spending is not charted");
        assert_eq!(s.income_by_category.len(), 1);
        assert_eq!(s.income_by_category[0].color.as_deref(), Some("#9ece6a"));
    }

    #[test]
    fn unknown_categories_get_a_placeholder_name() {
        let t = vec![tx("x", "expense", 100, "2026-03-01", Some("deleted"), "a")];
        let s = summarize(&t, &[], "2026-03-01", "2026-03-31");
        assert_eq!(s.expenses_by_category[0].name, "Unknown");
    }

    #[test]
    fn savings_rate_rounds_half_up_and_handles_no_income() {
        let s = summarize(&sample(), &cats(), "2026-03-01", "2026-03-31");
        assert_eq!(s.savings_rate, 52, "(200000-95700)/200000 = 52.15%");
        let none = summarize(&sample(), &cats(), "2026-04-01", "2026-04-30");
        assert_eq!((none.total_income, none.savings_rate), (0, 0));
        let half = vec![tx("i", "income", 200, "2026-03-01", None, "a"), tx("e", "expense", 101, "2026-03-01", None, "a")];
        assert_eq!(summarize(&half, &[], "2026-03-01", "2026-03-31").savings_rate, 50, "49.5 rounds up");
    }

    #[test]
    fn overspending_gives_a_negative_rate() {
        let t = vec![tx("i", "income", 100, "2026-03-01", None, "a"), tx("e", "expense", 250, "2026-03-01", None, "a")];
        let s = summarize(&t, &[], "2026-03-01", "2026-03-31");
        assert_eq!((s.net, s.savings_rate), (-150, -150));
    }

    #[test]
    fn filters_combine_range_type_and_account() {
        let all = sample();
        let ids = |v: Vec<&Transaction>| v.into_iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        let (s, e) = ("2026-03-01", "2026-03-31");
        assert_eq!(ids(filter_transactions(&all, s, e, TypeFilter::All, None)).len(), 5);
        assert_eq!(ids(filter_transactions(&all, s, e, TypeFilter::Income, None)), vec!["pay"]);
        assert_eq!(ids(filter_transactions(&all, s, e, TypeFilter::Expense, Some("b"))), vec!["food1"]);
        assert!(filter_transactions(&all, s, e, TypeFilter::Income, Some("b")).is_empty());
        assert_eq!(ids(filter_transactions(&all, "0000-01-01", "9999-12-31", TypeFilter::All, None)).len(), 6);
    }

    #[test]
    fn type_filter_parses_values() {
        assert_eq!(TypeFilter::from_value("income"), TypeFilter::Income);
        assert_eq!(TypeFilter::from_value("expense"), TypeFilter::Expense);
        assert_eq!(TypeFilter::from_value("all"), TypeFilter::All);
        assert_eq!(TypeFilter::from_value("???"), TypeFilter::All);
    }
}
