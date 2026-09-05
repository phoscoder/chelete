import { invoke } from "@tauri-apps/api/core";
import type {
  Account,
  Category,
  Transaction,
  Subscription,
  Overview,
  OmarchyTheme,
  CreateAccountRequest,
  CreateTransactionRequest,
  CreateCategoryRequest,
  CreateSubscriptionRequest,
  CsvMapping,
  CsvPreview,
  ImportOptions,
  ImportResult,
} from "../types";

export const api = {
  // Accounts
  getAccounts: () => invoke<Account[]>("get_accounts"),
  createAccount: (request: CreateAccountRequest) =>
    invoke<Account>("create_account", { request }),
  updateAccount: (request: { id: string } & Partial<Account>) =>
    invoke<Account>("update_account", { request }),
  deleteAccount: (id: string) =>
    invoke<void>("delete_account", { id }),

  // Transactions
  getTransactions: () => invoke<Transaction[]>("get_transactions"),
  createTransaction: (request: CreateTransactionRequest) =>
    invoke<Transaction>("create_transaction", { request }),
  updateTransaction: (request: { id: string } & Partial<Transaction>) =>
    invoke<Transaction>("update_transaction", { request }),
  deleteTransaction: (id: string) =>
    invoke<void>("delete_transaction", { id }),
  deleteTransactions: (ids: string[]) =>
    invoke<void>("delete_transactions", { ids }),

  // Categories
  getCategories: () => invoke<Category[]>("get_categories"),
  createCategory: (request: CreateCategoryRequest) =>
    invoke<Category>("create_category", { request }),
  updateCategory: (request: { id: string } & Partial<Category>) =>
    invoke<Category>("update_category", { request }),
  deleteCategory: (id: string) =>
    invoke<void>("delete_category", { id }),

  // Subscriptions
  getSubscriptions: () => invoke<Subscription[]>("get_subscriptions"),
  createSubscription: (request: CreateSubscriptionRequest) =>
    invoke<Subscription>("create_subscription", { request }),
  updateSubscription: (request: { id: string } & Partial<Subscription>) =>
    invoke<Subscription>("update_subscription", { request }),
  deleteSubscription: (id: string) =>
    invoke<void>("delete_subscription", { id }),

  // Overview
  getOverview: () => invoke<Overview>("get_overview"),

  // Theme
  getOmarchyTheme: () => invoke<OmarchyTheme>("get_omarchy_theme"),

  // Import
  openCsvFileDialog: () => invoke<string | null>("open_csv_file_dialog"),
  previewCsvImport: (path: string, mapping: CsvMapping) =>
    invoke<CsvPreview>("preview_csv_import", { path, mapping }),
  importTransactions: (
    path: string,
    mapping: CsvMapping,
    options: ImportOptions
  ) => invoke<ImportResult>("import_transactions", { path, mapping, options }),
};
