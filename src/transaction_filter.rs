//! Search, filters and sort order for the transactions list. Works on the
//! loaded transactions, so it composes with the date filter and paging.
use crate::commands::{Account, Category, Transaction};
use std::cmp::Ordering;

/// Category filter value that matches transactions without a category.
pub const UNCATEGORIZED: &str = "none";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    /// Words that must all appear in the description, merchant, notes,
    /// category, account or amount. Case-insensitive.
    pub search: String,
    /// `income` or `expense`.
    pub kind: Option<String>,
    pub account_id: Option<String>,
    /// A category id, or [`UNCATEGORIZED`].
    pub category_id: Option<String>,
}

impl TransactionFilter {
    pub fn is_empty(&self) -> bool {
        self.search.trim().is_empty() && self.kind.is_none() && self.account_id.is_none() && self.category_id.is_none()
    }

    pub fn matches(&self, t: &Transaction, accounts: &[Account], categories: &[Category]) -> bool {
        if self.kind.as_ref().is_some_and(|k| *k != t.transaction_type) {
            return false;
        }
        if self.account_id.as_ref().is_some_and(|a| *a != t.account_id) {
            return false;
        }
        match self.category_id.as_deref() {
            None => {}
            Some(UNCATEGORIZED) => {
                if category_name(t, categories).is_some() {
                    return false;
                }
            }
            Some(id) => {
                if t.category_id.as_deref() != Some(id) {
                    return false;
                }
            }
        }
        let terms: Vec<String> = self.search.split_whitespace().map(str::to_lowercase).collect();
        if terms.is_empty() {
            return true;
        }
        let haystack = [
            Some(t.description.as_str()),
            t.merchant.as_deref(),
            t.notes.as_deref(),
            category_name(t, categories),
            account_name(t, accounts),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();
        let amount = amount_text(t.amount);
        terms.iter().all(|term| {
            haystack.contains(term.as_str()) || {
                let digits: String = term.trim_start_matches('$').chars().filter(|c| *c != ',').collect();
                !digits.is_empty() && amount.contains(&digits)
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Date,
    Description,
    Category,
    Account,
    Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub key: SortKey,
    pub descending: bool,
}

impl Default for Sort {
    /// Newest first, the order transactions are loaded in.
    fn default() -> Self {
        Self { key: SortKey::Date, descending: true }
    }
}

impl Sort {
    /// The order after clicking a column header: the same column flips
    /// direction, a new one starts newest/largest first for dates and amounts
    /// and A–Z for text.
    pub fn toggled(self, key: SortKey) -> Self {
        if key == self.key {
            Self { key, descending: !self.descending }
        } else {
            Self { key, descending: matches!(key, SortKey::Date | SortKey::Amount) }
        }
    }

    /// Arrow for the column header of `key`, empty for the other columns.
    pub fn arrow(self, key: SortKey) -> &'static str {
        match (key == self.key, self.descending) {
            (false, _) => "",
            (true, true) => " ↓",
            (true, false) => " ↑",
        }
    }

    fn compare(self, a: &Transaction, b: &Transaction, accounts: &[Account], categories: &[Category]) -> Ordering {
        let text = |s: Option<&str>| s.unwrap_or("").to_lowercase();
        let order = match self.key {
            SortKey::Date => a.transaction_date.cmp(&b.transaction_date),
            SortKey::Description => text(Some(&a.description)).cmp(&text(Some(&b.description))),
            SortKey::Category => text(category_name(a, categories)).cmp(&text(category_name(b, categories))),
            SortKey::Account => text(account_name(a, accounts)).cmp(&text(account_name(b, accounts))),
            SortKey::Amount => signed(a).cmp(&signed(b)),
        };
        if self.descending {
            order.reverse()
        } else {
            order
        }
    }
}

/// The transactions that pass `filter`, in `sort` order. Ties keep their
/// loaded order (newest first).
pub fn apply<'a>(
    transactions: impl IntoIterator<Item = &'a Transaction>,
    filter: &TransactionFilter,
    sort: Sort,
    accounts: &[Account],
    categories: &[Category],
) -> Vec<&'a Transaction> {
    let mut rows: Vec<&Transaction> =
        transactions.into_iter().filter(|t| filter.matches(t, accounts, categories)).collect();
    if sort != Sort::default() {
        rows.sort_by(|a, b| sort.compare(a, b, accounts, categories));
    }
    rows
}

fn category_name<'a>(t: &Transaction, categories: &'a [Category]) -> Option<&'a str> {
    let id = t.category_id.as_deref()?;
    categories.iter().find(|c| c.id == id).map(|c| c.name.as_str())
}

fn account_name<'a>(t: &Transaction, accounts: &'a [Account]) -> Option<&'a str> {
    accounts.iter().find(|a| a.id == t.account_id).map(|a| a.name.as_str())
}

fn signed(t: &Transaction) -> i64 {
    if t.transaction_type == "expense" {
        -t.amount
    } else {
        t.amount
    }
}

fn amount_text(cents: i64) -> String {
    let abs = cents.unsigned_abs();
    format!("{}.{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(id: &str, date: &str, kind: &str, amount: i64, description: &str, account: &str, category: Option<&str>) -> Transaction {
        Transaction {
            id: id.into(),
            account_id: account.into(),
            category_id: category.map(String::from),
            transaction_type: kind.into(),
            amount,
            currency: "USD".into(),
            description: description.into(),
            merchant: None,
            notes: None,
            transaction_date: date.into(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn account(id: &str, name: &str) -> Account {
        Account {
            id: id.into(),
            name: name.into(),
            account_type: "bank".into(),
            currency: "USD".into(),
            balance: 0,
            color: None,
            icon: None,
            is_active: true,
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn category(id: &str, name: &str) -> Category {
        Category {
            id: id.into(),
            name: name.into(),
            parent_id: None,
            category_type: "expense".into(),
            icon: None,
            color: None,
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn fixture() -> (Vec<Transaction>, Vec<Account>, Vec<Category>) {
        let mut coffee = tx("1", "2026-03-03", "expense", 450, "Coffee", "bank", Some("food"));
        coffee.merchant = Some("Blue Bottle".into());
        let txs = vec![
            coffee,
            tx("2", "2026-03-02", "income", 250000, "Salary", "bank", Some("pay")),
            tx("3", "2026-03-01", "expense", 125050, "Rent", "cash", None),
            tx("4", "2026-02-28", "expense", 2000, "Lunch", "cash", Some("gone")),
        ];
        let accounts = vec![account("bank", "Main Bank"), account("cash", "Wallet")];
        let categories = vec![category("food", "Food"), category("pay", "Paycheck")];
        (txs, accounts, categories)
    }

    fn ids(rows: &[&Transaction]) -> Vec<String> {
        rows.iter().map(|t| t.id.clone()).collect()
    }

    fn run(filter: &TransactionFilter, sort: Sort) -> Vec<String> {
        let (txs, accounts, categories) = fixture();
        ids(&apply(&txs, filter, sort, &accounts, &categories))
    }

    fn search(text: &str) -> Vec<String> {
        run(&TransactionFilter { search: text.into(), ..Default::default() }, Sort::default())
    }

    #[test]
    fn an_empty_filter_keeps_everything_in_order() {
        assert!(TransactionFilter::default().is_empty());
        assert_eq!(run(&TransactionFilter::default(), Sort::default()), ["1", "2", "3", "4"]);
    }

    #[test]
    fn search_looks_at_every_text_field() {
        assert_eq!(search("coffee"), ["1"], "description, any case");
        assert_eq!(search("bottle"), ["1"], "merchant");
        assert_eq!(search("paycheck"), ["2"], "category name");
        assert_eq!(search("wallet"), ["3", "4"], "account name");
        assert_eq!(search("wallet lunch"), ["4"], "every word has to match");
        assert_eq!(search("nothing"), Vec::<String>::new());
    }

    #[test]
    fn search_matches_amounts() {
        assert_eq!(search("4.50"), ["1"]);
        assert_eq!(search("$1,250"), ["3"]);
        assert_eq!(search("2500"), ["2"]);
    }

    #[test]
    fn filters_by_type_account_and_category() {
        let kind = TransactionFilter { kind: Some("income".into()), ..Default::default() };
        assert_eq!(run(&kind, Sort::default()), ["2"]);
        let acct = TransactionFilter { account_id: Some("cash".into()), ..Default::default() };
        assert_eq!(run(&acct, Sort::default()), ["3", "4"]);
        let cat = TransactionFilter { category_id: Some("food".into()), ..Default::default() };
        assert_eq!(run(&cat, Sort::default()), ["1"]);
    }

    #[test]
    fn uncategorized_includes_deleted_categories() {
        let filter = TransactionFilter { category_id: Some(UNCATEGORIZED.into()), ..Default::default() };
        assert_eq!(run(&filter, Sort::default()), ["3", "4"]);
    }

    #[test]
    fn sorts_by_each_column() {
        let all = TransactionFilter::default();
        let by = |key, descending| run(&all, Sort { key, descending });
        assert_eq!(by(SortKey::Date, false), ["4", "3", "2", "1"]);
        assert_eq!(by(SortKey::Description, false), ["1", "4", "3", "2"]);
        assert_eq!(by(SortKey::Amount, true), ["2", "1", "4", "3"], "income first, biggest expense last");
        assert_eq!(by(SortKey::Account, false), ["1", "2", "3", "4"]);
        assert_eq!(by(SortKey::Category, false), ["3", "4", "1", "2"], "uncategorized first");
    }

    #[test]
    fn clicking_a_header_toggles_the_order() {
        let sort = Sort::default();
        assert_eq!(sort.toggled(SortKey::Date), Sort { key: SortKey::Date, descending: false });
        assert_eq!(sort.toggled(SortKey::Description), Sort { key: SortKey::Description, descending: false });
        assert_eq!(sort.toggled(SortKey::Amount), Sort { key: SortKey::Amount, descending: true });
        assert_eq!(sort.arrow(SortKey::Date), " ↓");
        assert_eq!(sort.toggled(SortKey::Date).arrow(SortKey::Date), " ↑");
        assert_eq!(sort.arrow(SortKey::Amount), "");
    }
}
