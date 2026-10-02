//! Rules the CSV import dialog applies on top of the parser: column
//! assignment, readiness, duplicate flagging and remembered mappings.
use super::{AmountSource, CsvField, CsvMapping, ParsedRow};
use std::collections::HashMap;
use std::path::Path;

/// Map `header` to `field`, or unmap it. A field can only come from one
/// column, so any other column using it is released.
pub fn set_column(mapping: &mut CsvMapping, header: &str, field: Option<CsvField>) {
    match field {
        None => {
            mapping.columns.remove(header);
        }
        Some(field) => {
            mapping.columns.retain(|_, f| *f != field);
            mapping.columns.insert(header.to_string(), field);
        }
    }
}

/// Date, description, an amount, and some way to tell income from expense.
pub fn is_valid_for_preview(mapping: &CsvMapping) -> bool {
    let has = |f| mapping.field_column(f).is_some();
    let has_amount = has(CsvField::Amount) || has(CsvField::IncomeAmount) || has(CsvField::ExpenseAmount);
    let has_type = has(CsvField::Type)
        || has(CsvField::IncomeAmount)
        || has(CsvField::ExpenseAmount)
        || mapping.default_transaction_type.is_some();
    has(CsvField::Date) && has(CsvField::Description) && has_amount && has_type
}

fn effective_account(row: &ParsedRow, mapping: &CsvMapping) -> Option<String> {
    if let Some(account) = row.account.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
        return Some(account.to_lowercase());
    }
    mapping.default_account_id.as_ref().map(|id| format!("__default:{id}"))
}

/// A row repeats an earlier row: same date, amount, description and account.
/// Rows with no account source are never flagged, or every blank-account row
/// would look identical.
pub fn is_duplicate_preview(row: &ParsedRow, all: &[ParsedRow], mapping: &CsvMapping) -> bool {
    let (Some(date), Some(amount), Some(description)) = (&row.date, row.amount_cents, &row.description) else {
        return false;
    };
    let Some(account) = effective_account(row, mapping) else {
        return false;
    };
    let description = description.trim().to_lowercase();
    all.iter().any(|other| {
        other.row_index < row.row_index
            && other.date.as_ref() == Some(date)
            && other.amount_cents == Some(amount)
            && other.description.as_deref().map(|d| d.trim().to_lowercase()).as_deref() == Some(description.as_str())
            && effective_account(other, mapping).as_deref() == Some(account.as_str())
    })
}

pub fn flag_duplicates(rows: &[ParsedRow], mapping: &CsvMapping) -> Vec<bool> {
    rows.iter().map(|r| is_duplicate_preview(r, rows, mapping)).collect()
}

/// Rows that would import: no errors and not a duplicate.
pub fn valid_count(rows: &[ParsedRow], duplicates: &[bool]) -> usize {
    rows.iter()
        .zip(duplicates)
        .filter(|(r, dup)| r.errors.is_empty() && !**dup)
        .count()
}

/// What the preview's Type column shows. Only separate income/expense columns
/// tell us the type before import.
pub fn type_label(row: &ParsedRow) -> &'static str {
    match row.amount_source {
        Some(AmountSource::IncomeColumn) => "income",
        Some(AmountSource::ExpenseColumn) => "expense",
        _ => "—",
    }
}

/// Identifies a CSV layout by its headers, ignoring case and punctuation.
pub fn mapping_key(headers: &[String]) -> String {
    headers
        .iter()
        .map(|h| h.trim().to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>())
        .collect::<Vec<_>>()
        .join(",")
}

/// Column mappings remembered per CSV layout.
#[derive(Debug, Default)]
pub struct SavedMappings(HashMap<String, CsvMapping>);

impl SavedMappings {
    pub fn load_from(path: &Path) -> Self {
        let map = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self(map)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&self.0).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn get(&self, headers: &[String]) -> Option<&CsvMapping> {
        self.0.get(&mapping_key(headers))
    }

    pub fn insert(&mut self, headers: &[String], mapping: CsvMapping) {
        self.0.insert(mapping_key(headers), mapping);
    }

    pub fn default_path() -> std::path::PathBuf {
        crate::database::data_dir().join("csv-mappings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::{parse_csv_preview, parse_csv_rows};

    fn headers(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn row(index: usize, date: &str, desc: &str, cents: i64, account: Option<&str>) -> ParsedRow {
        ParsedRow {
            row_index: index,
            date: Some(date.into()),
            description: Some(desc.into()),
            merchant: None,
            transaction_type: None,
            amount_cents: Some(cents),
            amount_source: None,
            currency: None,
            category: None,
            account: account.map(String::from),
            notes: None,
            errors: vec![],
        }
    }

    #[test]
    fn a_field_comes_from_one_column_only() {
        let mut m = CsvMapping::from_headers(&headers(&["Date", "Memo", "Amount"]));
        set_column(&mut m, "Memo", Some(CsvField::Amount));
        assert_eq!(m.columns.get("Memo"), Some(&CsvField::Amount));
        assert_eq!(m.columns.get("Amount"), None, "previous amount column released");
        assert_eq!(m.field_column(CsvField::Amount), Some("Memo"));
        set_column(&mut m, "Memo", None);
        assert_eq!(m.field_column(CsvField::Amount), None);
    }

    #[test]
    fn readiness_needs_date_description_amount_and_a_type_source() {
        let mut m = CsvMapping::default();
        assert!(!is_valid_for_preview(&m));
        set_column(&mut m, "d", Some(CsvField::Date));
        set_column(&mut m, "p", Some(CsvField::Description));
        set_column(&mut m, "a", Some(CsvField::Amount));
        assert!(!is_valid_for_preview(&m), "amount alone does not say income or expense");
        m.default_transaction_type = Some("expense".into());
        assert!(is_valid_for_preview(&m));
        m.default_transaction_type = None;
        set_column(&mut m, "t", Some(CsvField::Type));
        assert!(is_valid_for_preview(&m));
    }

    #[test]
    fn separate_income_and_expense_columns_are_enough() {
        let mut m = CsvMapping::default();
        set_column(&mut m, "d", Some(CsvField::Date));
        set_column(&mut m, "p", Some(CsvField::Description));
        set_column(&mut m, "in", Some(CsvField::IncomeAmount));
        assert!(is_valid_for_preview(&m));
    }

    #[test]
    fn duplicates_match_on_date_amount_description_and_account() {
        let m = CsvMapping::default();
        let rows = vec![
            row(1, "2026-03-01", "Coffee", 450, Some("Main")),
            row(2, "2026-03-01", "  coffee ", 450, Some("main")),
            row(3, "2026-03-01", "Coffee", 451, Some("Main")),
            row(4, "2026-03-02", "Coffee", 450, Some("Main")),
            row(5, "2026-03-01", "Coffee", 450, Some("Other")),
        ];
        assert_eq!(flag_duplicates(&rows, &m), vec![false, true, false, false, false]);
        assert_eq!(valid_count(&rows, &flag_duplicates(&rows, &m)), 4);
    }

    #[test]
    fn rows_without_an_account_source_are_never_duplicates() {
        let m = CsvMapping::default();
        let rows = vec![row(1, "2026-03-01", "A", 1, None), row(2, "2026-03-01", "A", 1, None)];
        assert_eq!(flag_duplicates(&rows, &m), vec![false, false]);
    }

    #[test]
    fn a_default_account_makes_blank_accounts_comparable() {
        let mut m = CsvMapping::default();
        m.default_account_id = Some("acc-1".into());
        let rows = vec![row(1, "2026-03-01", "A", 1, None), row(2, "2026-03-01", "A", 1, Some(""))];
        assert_eq!(flag_duplicates(&rows, &m), vec![false, true]);
    }

    #[test]
    fn rows_with_errors_do_not_count_as_valid() {
        let m = CsvMapping::default();
        let mut bad = row(1, "2026-03-01", "A", 1, Some("x"));
        bad.errors.push("bad date".into());
        let rows = vec![bad, row(2, "2026-03-02", "B", 2, Some("x"))];
        assert_eq!(valid_count(&rows, &flag_duplicates(&rows, &m)), 1);
    }

    #[test]
    fn type_label_reflects_split_amount_columns() {
        let mut r = row(1, "d", "x", 1, None);
        assert_eq!(type_label(&r), "—");
        r.amount_source = Some(AmountSource::IncomeColumn);
        assert_eq!(type_label(&r), "income");
        r.amount_source = Some(AmountSource::ExpenseColumn);
        assert_eq!(type_label(&r), "expense");
    }

    #[test]
    fn keys_ignore_case_and_punctuation() {
        assert_eq!(mapping_key(&headers(&["Transaction Date", "Amount ($)"])), "transactiondate,amount");
        assert_eq!(mapping_key(&headers(&["transaction_date", "AMOUNT"])), "transactiondate,amount");
    }

    #[test]
    fn saved_mappings_round_trip_on_disk() {
        let dir = std::env::temp_dir().join(format!("chelete-mappings-{}", std::process::id()));
        let path = dir.join("m.json");
        let hs = headers(&["Date", "Memo", "Amount"]);
        let mut saved = SavedMappings::load_from(&path);
        assert!(saved.get(&hs).is_none(), "missing file starts empty");
        let mut mapping = CsvMapping::from_headers(&hs);
        mapping.default_account_id = Some("acc".into());
        saved.insert(&hs, mapping);
        saved.save_to(&path).unwrap();

        let loaded = SavedMappings::load_from(&path);
        let got = loaded.get(&headers(&["date", "MEMO", "amount"])).expect("same layout, different case");
        assert_eq!(got.default_account_id.as_deref(), Some("acc"));
        assert_eq!(got.columns.get("Date"), Some(&CsvField::Date));
        assert!(loaded.get(&headers(&["Other"])).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupt_saved_file_is_ignored() {
        let dir = std::env::temp_dir().join(format!("chelete-mappings-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("m.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(SavedMappings::load_from(&path).get(&headers(&["a"])).is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn works_end_to_end_on_a_real_csv() {
        let dir = std::env::temp_dir().join(format!("chelete-csv-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bank.csv");
        std::fs::write(
            &path,
            "Date,Payee,Debit,Credit,Account\n2026-03-01,Coffee,4.50,,Main\n2026-03-01,Coffee,4.50,,Main\n2026-03-02,Salary,,2000.00,Main\n",
        )
        .unwrap();
        let p = path.to_string_lossy().to_string();

        let first = parse_csv_preview(&p, &CsvMapping::default()).unwrap();
        let mapping = CsvMapping::from_headers(&first.headers);
        assert!(is_valid_for_preview(&mapping), "headers are recognised automatically");

        let rows = parse_csv_rows(&p, &mapping).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(flag_duplicates(&rows, &mapping), vec![false, true, false]);
        assert_eq!(type_label(&rows[2]), "income");
        std::fs::remove_dir_all(&dir).ok();
    }
}
