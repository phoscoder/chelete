import { describe, it, expect, vi, beforeEach } from "vitest";
import { api } from "../api";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("api.getAccounts", () => {
  it("calls invoke with correct command", async () => {
    const mockAccounts = [
      { id: "1", name: "Checking", balance: 500000 },
      { id: "2", name: "Savings", balance: 1500000 },
    ];
    mockInvoke.mockResolvedValue(mockAccounts);

    const result = await api.getAccounts();

    expect(mockInvoke).toHaveBeenCalledWith("get_accounts");
    expect(result).toEqual(mockAccounts);
  });
});

describe("api.createAccount", () => {
  it("calls invoke with correct command and request", async () => {
    const request = {
      name: "New Account",
      account_type: "bank",
      currency: "USD",
      balance: 100000,
    };
    const mockAccount = { id: "new-id", ...request, is_active: true };
    mockInvoke.mockResolvedValue(mockAccount);

    const result = await api.createAccount(request);

    expect(mockInvoke).toHaveBeenCalledWith("create_account", { request });
    expect(result).toEqual(mockAccount);
  });
});

describe("api.deleteAccount", () => {
  it("calls invoke with correct command and id", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await api.deleteAccount("account-123");

    expect(mockInvoke).toHaveBeenCalledWith("delete_account", {
      id: "account-123",
    });
  });
});

describe("api.getTransactions", () => {
  it("calls invoke with correct command", async () => {
    mockInvoke.mockResolvedValue([]);

    const result = await api.getTransactions();

    expect(mockInvoke).toHaveBeenCalledWith("get_transactions");
    expect(result).toEqual([]);
  });
});

describe("api.createTransaction", () => {
  it("calls invoke with correct command and request", async () => {
    const request = {
      account_id: "acc-1",
      transaction_type: "expense",
      amount: 5000,
      currency: "USD",
      description: "Groceries",
      transaction_date: "2026-03-15",
    };
    const mockTxn = { id: "txn-1", ...request };
    mockInvoke.mockResolvedValue(mockTxn);

    const result = await api.createTransaction(request);

    expect(mockInvoke).toHaveBeenCalledWith("create_transaction", { request });
    expect(result).toEqual(mockTxn);
  });
});

describe("api.getCategories", () => {
  it("calls invoke with correct command", async () => {
    const mockCategories = [{ id: "cat-1", name: "Food" }];
    mockInvoke.mockResolvedValue(mockCategories);

    const result = await api.getCategories();

    expect(mockInvoke).toHaveBeenCalledWith("get_categories");
    expect(result).toEqual(mockCategories);
  });
});

describe("api.createCategory", () => {
  it("calls invoke with correct command and request", async () => {
    const request = {
      name: "New Category",
      category_type: "expense",
      icon: "star",
      color: "#ff0000",
    };
    const mockCategory = { id: "cat-new", ...request };
    mockInvoke.mockResolvedValue(mockCategory);

    const result = await api.createCategory(request);

    expect(mockInvoke).toHaveBeenCalledWith("create_category", { request });
    expect(result).toEqual(mockCategory);
  });
});

describe("api.getOverview", () => {
  it("calls invoke with correct command", async () => {
    const mockOverview = {
      total_balance: 2000000,
      total_income: 450000,
      total_expenses: 120000,
      accounts: [],
      recent_transactions: [],
      category_spending: [],
    };
    mockInvoke.mockResolvedValue(mockOverview);

    const result = await api.getOverview();

    expect(mockInvoke).toHaveBeenCalledWith("get_overview");
    expect(result).toEqual(mockOverview);
  });
});

describe("api.getOmarchyTheme", () => {
  it("calls invoke with correct command", async () => {
    const mockTheme = {
      name: "default",
      background: "#1a1b26",
      foreground: "#c0caf5",
      accent: "#7aa2f7",
      colors: {},
    };
    mockInvoke.mockResolvedValue(mockTheme);

    const result = await api.getOmarchyTheme();

    expect(mockInvoke).toHaveBeenCalledWith("get_omarchy_theme");
    expect(result).toEqual(mockTheme);
  });
});
