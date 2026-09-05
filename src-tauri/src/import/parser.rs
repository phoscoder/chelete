use super::mapping::{CsvField, CsvMapping};
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmountSource {
    IncomeColumn,
    ExpenseColumn,
    AmountColumn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRow {
    pub row_index: usize,
    pub date: Option<String>,
    pub description: Option<String>,
    pub merchant: Option<String>,
    pub transaction_type: Option<String>,
    pub amount_cents: Option<i64>,
    pub amount_source: Option<AmountSource>,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub account: Option<String>,
    pub notes: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvPreview {
    pub headers: Vec<String>,
    pub rows: Vec<ParsedRow>,
    pub total_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

const PREVIEW_ROWS: usize = 20;

/// Parse a CSV file and return a preview of the first rows.
pub fn parse_csv_preview(path: &str, mapping: &CsvMapping) -> Result<CsvPreview, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    let mut total_rows = 0usize;

    for (idx, result) in reader.records().enumerate() {
        total_rows += 1;
        if idx >= PREVIEW_ROWS {
            continue;
        }
        let record = result.map_err(|e| e.to_string())?;
        let parsed = parse_record(idx + 2, &headers, &record, mapping);
        rows.push(parsed);
    }

    Ok(CsvPreview {
        headers,
        rows,
        total_rows,
    })
}

/// Parse every row of a CSV file.
pub fn parse_csv_rows(path: &str, mapping: &CsvMapping) -> Result<Vec<ParsedRow>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows = Vec::new();
    for (idx, result) in reader.records().enumerate() {
        let record = result.map_err(|e| e.to_string())?;
        rows.push(parse_record(idx + 2, &headers, &record, mapping));
    }
    Ok(rows)
}

fn parse_record(
    row_index: usize,
    headers: &[String],
    record: &StringRecord,
    mapping: &CsvMapping,
) -> ParsedRow {
    let cells: HashMap<&str, &str> = headers
        .iter()
        .zip(record.iter())
        .map(|(h, v)| (h.as_str(), v))
        .collect();

    let mut errors = Vec::new();

    let date = mapping
        .field_column(CsvField::Date)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
    let normalized_date = date.as_deref().and_then(parse_date).map(|d| d.to_string());
    if date.is_some() && normalized_date.is_none() {
        errors.push(format!(
            "Could not parse date '{}' (expected YYYY-MM-DD, MM/DD/YYYY, or DD/MM/YYYY).",
            date.as_ref().unwrap()
        ));
    }

    let description = mapping
        .field_column(CsvField::Description)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    let merchant = mapping
        .field_column(CsvField::Merchant)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    let notes = mapping
        .field_column(CsvField::Notes)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    let transaction_type = mapping
        .field_column(CsvField::Type)
        .and_then(|c| cells.get(c).copied())
        .map(|v| normalize_type(v.trim(), mapping));

    let (amount_cents, amount_source, amount_error) = compute_amount(&cells, mapping);
    if let Some(err) = amount_error {
        errors.push(err);
    }

    let currency = mapping
        .field_column(CsvField::Currency)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim().to_uppercase();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .or_else(|| mapping.default_currency.clone());

    let category = mapping
        .field_column(CsvField::Category)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    let account = mapping
        .field_column(CsvField::Account)
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

    if normalized_date.is_none() {
        errors.push("Date is required.".to_string());
    }
    if description.is_none() {
        errors.push("Description is required.".to_string());
    }
    if amount_cents.is_none() && !errors.iter().any(|e| e.contains("amount")) {
        errors.push("Amount is required.".to_string());
    }
    let has_type_source = transaction_type.is_some()
        || amount_source == Some(AmountSource::IncomeColumn)
        || amount_source == Some(AmountSource::ExpenseColumn);
    if !has_type_source && mapping.field_column(CsvField::Type).is_some() {
        errors.push("Transaction type is required.".to_string());
    }

    ParsedRow {
        row_index,
        date: normalized_date.or(date),
        description,
        merchant,
        transaction_type,
        amount_cents,
        amount_source,
        currency,
        category,
        account,
        notes,
        errors,
    }
}

fn compute_amount(
    cells: &HashMap<&str, &str>,
    mapping: &CsvMapping,
) -> (Option<i64>, Option<AmountSource>, Option<String>) {
    // If separate income/expense amount columns exist, use them.
    let income_col = mapping.field_column(CsvField::IncomeAmount);
    let expense_col = mapping.field_column(CsvField::ExpenseAmount);

    let income_raw = income_col
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
    let expense_raw = expense_col
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    if let (Some(inc), Some(exp)) = (income_raw, expense_raw) {
        match (parse_money(inc), parse_money(exp)) {
            (Ok(Some(inc_cents)), Ok(Some(exp_cents))) => {
                if inc_cents > 0 && exp_cents == 0 {
                    return (Some(inc_cents), Some(AmountSource::IncomeColumn), None);
                } else if exp_cents > 0 && inc_cents == 0 {
                    return (Some(exp_cents), Some(AmountSource::ExpenseColumn), None);
                } else if inc_cents == 0 && exp_cents == 0 {
                    return (None, None, Some("Amount is required.".to_string()));
                } else {
                    return (
                        None,
                        None,
                        Some(format!(
                            "Both income ({}) and expense ({}) amounts have values.",
                            inc, exp
                        )),
                    );
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                return (None, None, Some(format!("Invalid amount: {}", e)));
            }
            _ => {}
        }
    } else if let Some(inc) = income_raw {
        match parse_money(inc) {
            Ok(Some(cents)) if cents > 0 => {
                return (Some(cents), Some(AmountSource::IncomeColumn), None)
            }
            Ok(_) => return (None, None, Some("Income amount must be positive.".to_string())),
            Err(e) => return (None, None, Some(format!("Invalid income amount: {}", e))),
        }
    } else if let Some(exp) = expense_raw {
        match parse_money(exp) {
            Ok(Some(cents)) if cents > 0 => {
                return (Some(cents), Some(AmountSource::ExpenseColumn), None)
            }
            Ok(_) => return (None, None, Some("Expense amount must be positive.".to_string())),
            Err(e) => return (None, None, Some(format!("Invalid expense amount: {}", e))),
        }
    }

    // Single amount column.
    let amount_col = mapping.field_column(CsvField::Amount);
    let raw = amount_col
        .and_then(|c| cells.get(c).copied())
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    let Some(raw) = raw else {
        return (None, None, Some("Amount is required.".to_string()));
    };

    match parse_money(raw) {
        Ok(Some(cents)) => {
            // A negative amount in a single amount column implies expense when no type column
            // is mapped; return the absolute value with an expense source.
            if cents < 0 && mapping.field_column(CsvField::Type).is_none() {
                return (Some(-cents), Some(AmountSource::ExpenseColumn), None);
            }
            (Some(cents), Some(AmountSource::AmountColumn), None)
        }
        Ok(None) => (None, None, Some("Amount is required.".to_string())),
        Err(e) => (None, None, Some(format!("Invalid amount '{}': {}", raw, e))),
    }
}

/// Parse a monetary value and return the value in cents.
/// A leading minus sign or surrounding parentheses indicate a negative amount.
pub fn parse_money(value: &str) -> Result<Option<i64>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    // Detect accounting-style parentheses negative before stripping them.
    let in_parens = trimmed.starts_with('(') && trimmed.ends_with(')');
    let has_leading_minus = trimmed.starts_with('-');
    let negative = in_parens || has_leading_minus;

    // Strip currency symbols, whitespace, and sign characters.
    let cleaned: String = trimmed
        .chars()
        .filter(|c| !['$', '€', '£', '¥', '₹', '₩', ' ', '-', '(', ')'].contains(c))
        .collect();

    // Count separators to decide decimal convention.
    let comma_count = cleaned.matches(',').count();
    let dot_count = cleaned.matches('.').count();

    let normalized = if dot_count >= 1 && comma_count >= 1 {
        // Mixed separators: the rightmost one is the decimal separator.
        // e.g. "1,234.56" -> remove commas, keep dot.
        // e.g. "1.234,56" -> remove dots, convert comma to dot.
        if let Some(last_dot) = cleaned.rfind('.') {
            if let Some(last_comma) = cleaned.rfind(',') {
                if last_dot > last_comma {
                    cleaned.replace(',', "")
                } else {
                    cleaned.replace('.', "").replace(',', ".")
                }
            } else {
                cleaned.replace(',', "")
            }
        } else {
            cleaned.replace('.', "").replace(',', ".")
        }
    } else if comma_count == 1 && dot_count == 0 {
        // Single comma: could be decimal (European) or thousands (ambiguous).
        // Treat as decimal only if it has 1 or 2 digits after it, otherwise remove it.
        let parts: Vec<&str> = cleaned.split(',').collect();
        if parts.len() == 2 && parts[1].len() <= 2 {
            cleaned.replace(',', ".")
        } else {
            cleaned.replace(',', "")
        }
    } else if dot_count == 1 && comma_count == 0 {
        // Single dot: standard decimal separator.
        cleaned
    } else if dot_count >= 2 && comma_count == 0 {
        // Multiple dots: thousands separators using dots.
        cleaned.replace('.', "")
    } else if comma_count >= 2 && dot_count == 0 {
        // Multiple commas: thousands separators using commas.
        cleaned.replace(',', "")
    } else {
        cleaned
    };

    match normalized.parse::<f64>() {
        Ok(v) if v.is_finite() && v >= 0.0 => {
            let cents = (v * 100.0).round() as i64;
            if negative {
                Ok(Some(-cents))
            } else {
                Ok(Some(cents))
            }
        }
        Ok(_) => Err("Amount must be a finite number.".to_string()),
        Err(_) => Err(format!("Could not parse '{}' as a number.", value)),
    }
}

pub fn normalize_type(value: &str, mapping: &CsvMapping) -> String {
    let key = value.trim().to_lowercase();
    mapping
        .type_aliases
        .get(&key)
        .cloned()
        .unwrap_or_else(|| key.clone())
}

/// Try to parse a date into YYYY-MM-DD.
pub fn parse_date(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Already ISO.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    // US style.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%m/%d/%y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    // European style.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d/%m/%y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    // Dotted formats.
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%Y.%m.%d") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d.%m.%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_money_basic() {
        assert_eq!(parse_money("10.00").unwrap(), Some(1000));
        assert_eq!(parse_money("$1,234.56").unwrap(), Some(123456));
        assert_eq!(parse_money("(45.67)").unwrap(), Some(-4567));
        assert_eq!(parse_money("-89.10").unwrap(), Some(-8910));
        assert_eq!(parse_money("").unwrap(), None);
        assert_eq!(parse_money("   ").unwrap(), None);
    }

    #[test]
    fn test_parse_money_european() {
        assert_eq!(parse_money("1.234,56").unwrap(), Some(123456));
        assert_eq!(parse_money("12,34").unwrap(), Some(1234));
        assert_eq!(parse_money("1,234").unwrap(), Some(123400));
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2024-03-15"), Some("2024-03-15".to_string()));
        assert_eq!(parse_date("03/15/2024"), Some("2024-03-15".to_string()));
        assert_eq!(parse_date("15/03/2024"), Some("2024-03-15".to_string()));
        assert_eq!(parse_date("2024.03.15"), Some("2024-03-15".to_string()));
    }

    #[test]
    fn test_normalize_type() {
        let mut mapping = CsvMapping::default();
        mapping.type_aliases.insert("debit".to_string(), "expense".to_string());
        assert_eq!(normalize_type("Debit", &mapping), "expense");
        assert_eq!(normalize_type("Income", &mapping), "income");
    }

    #[test]
    fn test_parse_record_with_separate_amount_columns() {
        let headers = vec![
            "Date".to_string(),
            "Description".to_string(),
            "Debit".to_string(),
            "Credit".to_string(),
        ];
        let record = StringRecord::from(vec!["2024-03-15", "Paycheck", "", "1500.00"]);
        let mapping = CsvMapping::from_headers(&headers);
        let parsed = parse_record(2, &headers, &record, &mapping);
        assert_eq!(parsed.date, Some("2024-03-15".to_string()));
        assert_eq!(parsed.description, Some("Paycheck".to_string()));
        assert_eq!(parsed.amount_cents, Some(150000));
        assert_eq!(parsed.amount_source, Some(AmountSource::IncomeColumn));
    }

    #[test]
    fn test_parse_record_with_type_column() {
        let headers = vec![
            "Date".to_string(),
            "Description".to_string(),
            "Amount".to_string(),
            "Type".to_string(),
        ];
        let record = StringRecord::from(vec!["2024-03-15", "Groceries", "45.67", "Debit"]);
        let mut mapping = CsvMapping::from_headers(&headers);
        mapping.default_account_id = Some("acct-1".to_string());
        let parsed = parse_record(2, &headers, &record, &mapping);
        assert_eq!(parsed.amount_cents, Some(4567));
        assert_eq!(parsed.transaction_type, Some("expense".to_string()));
    }

    #[test]
    fn test_parse_record_requires_account_default() {
        let headers = vec![
            "Date".to_string(),
            "Description".to_string(),
            "Amount".to_string(),
            "Type".to_string(),
        ];
        let record = StringRecord::from(vec!["2024-03-15", "Groceries", "45.67", "expense"]);
        let mapping = CsvMapping::from_headers(&headers);
        let parsed = parse_record(2, &headers, &record, &mapping);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    }
}
