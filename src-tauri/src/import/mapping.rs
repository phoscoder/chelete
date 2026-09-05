use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsvField {
    Date,
    Description,
    Merchant,
    Amount,
    IncomeAmount,
    ExpenseAmount,
    Type,
    Category,
    Account,
    Currency,
    Notes,
    Ignore,
}

impl CsvField {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            CsvField::Date => "Date",
            CsvField::Description => "Description",
            CsvField::Merchant => "Merchant",
            CsvField::Amount => "Amount",
            CsvField::IncomeAmount => "Income Amount",
            CsvField::ExpenseAmount => "Expense Amount",
            CsvField::Type => "Type (income/expense)",
            CsvField::Category => "Category",
            CsvField::Account => "Account",
            CsvField::Currency => "Currency",
            CsvField::Notes => "Notes",
            CsvField::Ignore => "Ignore",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> &'static [CsvField] {
        &[
            CsvField::Date,
            CsvField::Description,
            CsvField::Merchant,
            CsvField::Amount,
            CsvField::IncomeAmount,
            CsvField::ExpenseAmount,
            CsvField::Type,
            CsvField::Category,
            CsvField::Account,
            CsvField::Currency,
            CsvField::Notes,
            CsvField::Ignore,
        ]
    }

    #[allow(dead_code)]
    pub fn default_mapping_for_header(header: &str) -> Option<CsvField> {
        let h = header.trim().to_lowercase().replace([' ', '_', '-'], "");
        match h.as_str() {
            "date" | "transactiondate" | "transdate" | "posted" | "datetime" => {
                Some(CsvField::Date)
            }
            "description" | "payee" | "memo" | "details" | "narrative" => {
                Some(CsvField::Description)
            }
            "merchant" => Some(CsvField::Merchant),
            "amount" | "value" | "sum" => Some(CsvField::Amount),
            "credit" | "deposit" | "income" | "received" | "inflow" => {
                Some(CsvField::IncomeAmount)
            }
            "debit" | "withdrawal" | "payment" | "expense" | "outflow" => {
                Some(CsvField::ExpenseAmount)
            }
            "type" | "transactiontype" | "kind" => Some(CsvField::Type),
            "category" | "categoryname" => Some(CsvField::Category),
            "account" | "accountname" | "source" => Some(CsvField::Account),
            "currency" | "cur" => Some(CsvField::Currency),
            "notes" | "comment" | "note" => Some(CsvField::Notes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvMapping {
    /// Maps CSV header name -> Chelete field.
    pub columns: std::collections::HashMap<String, CsvField>,
    /// Default account to use when the CSV has no account column or the value is blank.
    pub default_account_id: Option<String>,
    /// Default category to use when the CSV has no category column or the value is blank.
    pub default_category_id: Option<String>,
    /// Default currency to use when the CSV has no currency column or the value is blank.
    pub default_currency: Option<String>,
    /// Default transaction type to use when the CSV has no type column and no separate
    /// income/expense amount columns.
    pub default_transaction_type: Option<String>,
    /// Type normalization: e.g. {"debit" => "expense", "credit" => "income"}
    pub type_aliases: std::collections::HashMap<String, String>,
}

impl Default for CsvMapping {
    fn default() -> Self {
        Self {
            columns: std::collections::HashMap::new(),
            default_account_id: None,
            default_category_id: None,
            default_currency: Some("USD".to_string()),
            default_transaction_type: None,
            type_aliases: std::collections::HashMap::new(),
        }
    }
}

impl CsvMapping {
    #[allow(dead_code)]
    pub fn from_headers(headers: &[String]) -> Self {
        let mut columns = std::collections::HashMap::new();
        for header in headers {
            if let Some(field) = CsvMapping::suggest_field(header) {
                columns.insert(header.clone(), field);
            }
        }
        let mut type_aliases = std::collections::HashMap::new();
        type_aliases.insert("debit".to_string(), "expense".to_string());
        type_aliases.insert("credit".to_string(), "income".to_string());
        type_aliases.insert("dr".to_string(), "expense".to_string());
        type_aliases.insert("cr".to_string(), "income".to_string());
        type_aliases.insert("in".to_string(), "income".to_string());
        type_aliases.insert("out".to_string(), "expense".to_string());
        type_aliases.insert("expense".to_string(), "expense".to_string());
        type_aliases.insert("income".to_string(), "income".to_string());
        Self {
            columns,
            default_account_id: None,
            default_category_id: None,
            default_currency: Some("USD".to_string()),
            default_transaction_type: None,
            type_aliases,
        }
    }

    #[allow(dead_code)]
    fn suggest_field(header: &str) -> Option<CsvField> {
        let h = header.trim().to_lowercase().replace([' ', '_', '-'], "");
        if h == "date"
            || h == "transactiondate"
            || h == "transdate"
            || h == "posted"
            || h == "datetime"
        {
            Some(CsvField::Date)
        } else if h == "description"
            || h == "payee"
            || h == "memo"
            || h == "details"
            || h == "narrative"
        {
            Some(CsvField::Description)
        } else if h == "merchant" {
            Some(CsvField::Merchant)
        } else if h == "amount" || h == "value" || h == "sum" {
            Some(CsvField::Amount)
        } else if h == "credit"
            || h == "deposit"
            || h == "income"
            || h == "received"
            || h == "inflow"
        {
            Some(CsvField::IncomeAmount)
        } else if h == "debit"
            || h == "withdrawal"
            || h == "payment"
            || h == "expense"
            || h == "outflow"
        {
            Some(CsvField::ExpenseAmount)
        } else if h == "type" || h == "transactiontype" || h == "kind" {
            Some(CsvField::Type)
        } else if h == "category" || h == "categoryname" {
            Some(CsvField::Category)
        } else if h == "account" || h == "accountname" || h == "source" {
            Some(CsvField::Account)
        } else if h == "currency" || h == "cur" {
            Some(CsvField::Currency)
        } else if h == "notes" || h == "comment" || h == "note" {
            Some(CsvField::Notes)
        } else {
            None
        }
    }

    pub fn field_column(&self, field: CsvField) -> Option<&str> {
        self.columns
            .iter()
            .find(|(_, f)| **f == field)
            .map(|(h, _)| h.as_str())
    }
}
