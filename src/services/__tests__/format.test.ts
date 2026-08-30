import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  formatMoney,
  formatMoneyShort,
  formatBalance,
  formatDate,
  formatDateFull,
  todayISO,
  currentMonth,
  accountTypeLabel,
} from "../format";

describe("formatMoney", () => {
  it("formats positive amount with + sign", () => {
    expect(formatMoney(1000)).toBe("+$10.00");
    expect(formatMoney(525000)).toBe("+$5,250.00");
  });

  it("formats negative amount with - sign", () => {
    expect(formatMoney(-1000)).toBe("-$10.00");
    expect(formatMoney(-120000)).toBe("-$1,200.00");
  });

  it("formats zero as positive", () => {
    expect(formatMoney(0)).toBe("+$0.00");
  });

  it("handles small amounts", () => {
    expect(formatMoney(50)).toBe("+$0.50");
    expect(formatMoney(-25)).toBe("-$0.25");
  });
});

describe("formatMoneyShort", () => {
  it("formats amounts under 1000 normally", () => {
    expect(formatMoneyShort(500)).toBe("$5.00");
    expect(formatMoneyShort(12345)).toBe("$123.45");
  });

  it("formats amounts >= 1000 with k suffix", () => {
    expect(formatMoneyShort(100000)).toBe("$1.0k");
    expect(formatMoneyShort(1500000)).toBe("$15.0k");
    expect(formatMoneyShort(5250000)).toBe("$52.5k");
  });
});

describe("formatBalance", () => {
  it("formats positive balance without sign", () => {
    expect(formatBalance(525000)).toBe("$5,250.00");
  });

  it("formats negative balance with minus", () => {
    expect(formatBalance(-120000)).toBe("$-1,200.00");
  });

  it("formats zero balance", () => {
    expect(formatBalance(0)).toBe("$0.00");
  });
});

describe("formatDate", () => {
  it("formats date string to short format", () => {
    const result = formatDate("2026-03-15");
    expect(result).toMatch(/Mar/);
    expect(result).toContain("15");
  });
});

describe("formatDateFull", () => {
  it("formats date string to full format", () => {
    const result = formatDateFull("2026-03-15");
    expect(result).toMatch(/Mar/);
    expect(result).toContain("15");
    expect(result).toContain("2026");
  });
});

describe("todayISO", () => {
  it("returns today in YYYY-MM-DD format", () => {
    const result = todayISO();
    expect(result).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    const today = new Date().toISOString().split("T")[0];
    expect(result).toBe(today);
  });
});

describe("currentMonth", () => {
  it("returns current month and year", () => {
    const result = currentMonth();
    const now = new Date();
    const monthName = now.toLocaleDateString("en-US", { month: "long" });
    const year = now.getFullYear().toString();
    expect(result).toContain(monthName);
    expect(result).toContain(year);
  });
});

describe("accountTypeLabel", () => {
  it("returns correct labels for known types", () => {
    expect(accountTypeLabel("cash")).toBe("Cash");
    expect(accountTypeLabel("bank")).toBe("Bank");
    expect(accountTypeLabel("savings")).toBe("Savings");
    expect(accountTypeLabel("credit_card")).toBe("Credit Card");
    expect(accountTypeLabel("mobile_money")).toBe("Mobile Money");
    expect(accountTypeLabel("investment")).toBe("Investment");
    expect(accountTypeLabel("other")).toBe("Other");
  });

  it("returns the type string for unknown types", () => {
    expect(accountTypeLabel("crypto")).toBe("crypto");
    expect(accountTypeLabel("custom")).toBe("custom");
  });
});
