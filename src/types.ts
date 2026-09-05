export interface Account {
  id: string;
  name: string;
  account_type: string;
  currency: string;
  balance: number;
  color: string | null;
  icon: string | null;
  is_active: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Category {
  id: string;
  name: string;
  parent_id: string | null;
  category_type: string;
  icon: string | null;
  color: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Transaction {
  id: string;
  account_id: string;
  category_id: string | null;
  transaction_type: string;
  amount: number;
  currency: string;
  description: string;
  merchant: string | null;
  notes: string | null;
  transaction_date: string;
  created_at: string;
  updated_at: string;
}

export interface Overview {
  total_balance: number;
  total_income: number;
  total_expenses: number;
  accounts: Account[];
  recent_transactions: Transaction[];
  category_spending: CategorySpending[];
}

export interface CategorySpending {
  category_id: string;
  category_name: string;
  spent: number;
  budget_limit: number | null;
}

export interface OmarchyTheme {
  name: string;
  background: string;
  foreground: string;
  accent: string;
  colors: Record<string, string>;
}

export type View =
  | "overview"
  | "transactions"
  | "accounts"
  | "categories"
  | "subscriptions"
  | "settings";

export interface CreateAccountRequest {
  name: string;
  account_type: string;
  currency: string;
  balance: number;
  color?: string;
  icon?: string;
}

export interface CreateTransactionRequest {
  account_id: string;
  category_id?: string;
  transaction_type: string;
  amount: number;
  currency: string;
  description: string;
  merchant?: string;
  notes?: string;
  transaction_date: string;
}

export interface CreateCategoryRequest {
  name: string;
  parent_id?: string;
  category_type: string;
  icon?: string;
  color?: string;
}

export interface Subscription {
  id: string;
  name: string;
  amount: number;
  currency: string;
  frequency: string;
  category_id: string | null;
  account_id: string | null;
  start_date: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateSubscriptionRequest {
  name: string;
  amount: number;
  currency: string;
  frequency: string;
  category_id?: string;
  account_id?: string;
  start_date?: string;
}

export type CsvField =
  | "date"
  | "description"
  | "merchant"
  | "amount"
  | "income_amount"
  | "expense_amount"
  | "type"
  | "category"
  | "account"
  | "currency"
  | "notes"
  | "ignore";

export interface CsvMapping {
  columns: Record<string, CsvField>;
  default_account_id?: string;
  default_category_id?: string;
  default_currency?: string;
  default_transaction_type?: "income" | "expense";
  type_aliases: Record<string, string>;
}

export interface ParsedRow {
  row_index: number;
  date?: string;
  description?: string;
  merchant?: string;
  transaction_type?: string;
  amount_cents?: number;
  amount_source?: "income_column" | "expense_column" | "amount_column";
  currency?: string;
  category?: string;
  account?: string;
  notes?: string;
  errors: string[];
}

export interface CsvPreview {
  headers: string[];
  rows: ParsedRow[];
  total_rows: number;
}

export interface ImportOptions {
  default_account_id?: string;
  default_category_id?: string;
  default_currency?: string;
  skip_duplicates: boolean;
}

export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

export interface ExportData {
  accounts: Account[];
  categories: Category[];
  transactions: Transaction[];
  subscriptions: Subscription[];
}
