//! Data export formats.
use crate::commands::{Account, Category, ExportData, Transaction};
use std::collections::HashMap;

fn escape_csv_field(value: &str) -> String {
    if value.contains(['"', ',', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn transactions_to_csv(transactions: &[Transaction], accounts: &[Account], categories: &[Category]) -> String {
    let account_names: HashMap<&str, &str> = accounts.iter().map(|a| (a.id.as_str(), a.name.as_str())).collect();
    let category_names: HashMap<&str, &str> = categories.iter().map(|c| (c.id.as_str(), c.name.as_str())).collect();

    let headers = [
        "id", "transaction_date", "description", "merchant", "transaction_type", "amount", "currency",
        "account_name", "category_name", "notes", "created_at", "updated_at",
    ];
    let mut lines = vec![headers.join(",")];
    for t in transactions {
        let fields = [
            t.id.clone(),
            t.transaction_date.clone(),
            t.description.clone(),
            t.merchant.clone().unwrap_or_default(),
            t.transaction_type.clone(),
            format!("{:.2}", t.amount as f64 / 100.0),
            t.currency.clone(),
            account_names.get(t.account_id.as_str()).map_or_else(|| t.account_id.clone(), |n| n.to_string()),
            t.category_id
                .as_deref()
                .map(|id| category_names.get(id).map_or_else(|| id.to_string(), |n| n.to_string()))
                .unwrap_or_default(),
            t.notes.clone().unwrap_or_default(),
            t.created_at.clone(),
            t.updated_at.clone(),
        ];
        lines.push(fields.iter().map(|f| escape_csv_field(f)).collect::<Vec<_>>().join(","));
    }
    lines.join("\n")
}

pub fn export_to_json(data: &ExportData) -> Result<String, String> {
    serde_json::to_string_pretty(data).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
}

impl ExportFormat {
    pub fn default_file_name(self) -> &'static str {
        match self {
            ExportFormat::Csv => "chelete-transactions.csv",
            ExportFormat::Json => "chelete-export.json",
        }
    }

    pub fn contents(self, data: &ExportData) -> Result<String, String> {
        match self {
            ExportFormat::Csv => Ok(transactions_to_csv(&data.transactions, &data.accounts, &data.categories)),
            ExportFormat::Json => export_to_json(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn txn(desc: &str, merchant: Option<&str>, category: Option<&str>) -> Transaction {
        Transaction {
            id: "t1".into(),
            account_id: "a1".into(),
            category_id: category.map(String::from),
            transaction_type: "expense".into(),
            amount: 1234,
            currency: "USD".into(),
            description: desc.into(),
            merchant: merchant.map(String::from),
            notes: None,
            transaction_date: "2026-03-09".into(),
            created_at: "c".into(),
            updated_at: "u".into(),
        }
    }

    fn account() -> Account {
        Account {
            id: "a1".into(),
            name: "Main".into(),
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

    fn category() -> Category {
        Category {
            id: "c1".into(),
            name: "Food".into(),
            parent_id: None,
            category_type: "expense".into(),
            icon: None,
            color: None,
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn csv_has_header_and_resolved_names() {
        let csv = transactions_to_csv(&[txn("Lunch", Some("Cafe"), Some("c1"))], &[account()], &[category()]);
        let mut lines = csv.lines();
        assert_eq!(
            lines.next().unwrap(),
            "id,transaction_date,description,merchant,transaction_type,amount,currency,account_name,category_name,notes,created_at,updated_at"
        );
        assert_eq!(lines.next().unwrap(), "t1,2026-03-09,Lunch,Cafe,expense,12.34,USD,Main,Food,,c,u");
        assert!(lines.next().is_none());
    }

    #[test]
    fn csv_quotes_commas_quotes_and_newlines() {
        let csv = transactions_to_csv(&[txn("Dinner, \"fancy\"\nlate", None, None)], &[account()], &[]);
        assert!(csv.contains("\"Dinner, \"\"fancy\"\"\nlate\""), "{csv}");
    }

    #[test]
    fn unknown_ids_fall_back_to_the_raw_id() {
        let csv = transactions_to_csv(&[txn("x", None, Some("gone"))], &[], &[]);
        assert!(csv.contains(",a1,gone,"), "{csv}");
    }

    #[test]
    fn empty_export_is_just_the_header() {
        assert_eq!(transactions_to_csv(&[], &[], &[]).lines().count(), 1);
    }

    #[test]
    fn json_round_trips() {
        let data = ExportData {
            accounts: vec![account()],
            categories: vec![category()],
            transactions: vec![txn("Lunch", None, None)],
            subscriptions: vec![],
        };
        let json = export_to_json(&data).unwrap();
        let back: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(back["transactions"][0]["description"], "Lunch");
        assert_eq!(back["accounts"][0]["name"], "Main");
    }

    #[test]
    fn file_names() {
        assert_eq!(ExportFormat::Csv.default_file_name(), "chelete-transactions.csv");
        assert_eq!(ExportFormat::Json.default_file_name(), "chelete-export.json");
    }
}
