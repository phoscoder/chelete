import { invoke } from "@tauri-apps/api/core";
import type {
  Account,
  Category,
  Transaction,
  Overview,
  OmarchyTheme,
  CreateAccountRequest,
  CreateTransactionRequest,
  CreateCategoryRequest,
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

  // Categories
  getCategories: () => invoke<Category[]>("get_categories"),
  createCategory: (request: CreateCategoryRequest) =>
    invoke<Category>("create_category", { request }),
  updateCategory: (request: { id: string } & Partial<Category>) =>
    invoke<Category>("update_category", { request }),
  deleteCategory: (id: string) =>
    invoke<void>("delete_category", { id }),

  // Overview
  getOverview: () => invoke<Overview>("get_overview"),

  // Theme
  getOmarchyTheme: () => invoke<OmarchyTheme>("get_omarchy_theme"),
};
