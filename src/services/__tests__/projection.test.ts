import { describe, it, expect } from "vitest";
import {
  computeProjection,
  periodLabel,
  normalizeSubscriptionToPeriod,
  PERIOD_LIMITS,
  type ProjectionInput,
} from "../projection";

const baseInput = (overrides: Partial<ProjectionInput> = {}): ProjectionInput => ({
  startingBalance: 0,
  income: 0,
  expense: 0,
  periodType: "months",
  periods: 10,
  ...overrides,
});

describe("computeProjection", () => {
  it("matches the canonical example: 1200 income, 100 expense, 10 months -> 11000", () => {
    const result = computeProjection(
      baseInput({ income: 120000, expense: 10000, periodType: "months", periods: 10 })
    );

    expect(result.summary.netPerPeriod).toBe(110000);
    expect(result.summary.finalBalance).toBe(1100000);
    expect(result.summary.totalIncrease).toBe(1100000);
    expect(result.points).toHaveLength(11);
    expect(result.points[0].label).toBe("Now");
    expect(result.points[0].balance).toBe(0);
    expect(result.points[10].label).toBe("10 months");
    expect(result.points[10].balance).toBe(1100000);
  });

  it("balances increase linearly by net per period", () => {
    const result = computeProjection(
      baseInput({ income: 300000, expense: 120000, periodType: "weeks", periods: 4 })
    );

    expect(result.points.map((p) => p.balance)).toEqual([0, 180000, 360000, 540000, 720000]);
    expect(result.points.map((p) => p.label)).toEqual([
      "Now",
      "1 week",
      "2 weeks",
      "3 weeks",
      "4 weeks",
    ]);
  });

  it("offsets by starting balance", () => {
    const result = computeProjection(
      baseInput({
        startingBalance: 500000,
        income: 100000,
        expense: 0,
        periodType: "months",
        periods: 3,
      })
    );

    expect(result.summary.finalBalance).toBe(800000);
    expect(result.summary.totalIncrease).toBe(300000);
    expect(result.points[0].balance).toBe(500000);
    expect(result.points[3].balance).toBe(800000);
  });

  it("produces a flat line when income equals expense", () => {
    const result = computeProjection(
      baseInput({ income: 50000, expense: 50000, periods: 5 })
    );

    expect(result.summary.netPerPeriod).toBe(0);
    expect(result.points.every((p) => p.balance === 0)).toBe(true);
  });

  it("declines when expenses exceed income", () => {
    const result = computeProjection(
      baseInput({ income: 20000, expense: 50000, periods: 4 })
    );

    expect(result.summary.netPerPeriod).toBe(-30000);
    expect(result.summary.finalBalance).toBe(-120000);
    expect(result.points.map((p) => p.balance)).toEqual([0, -30000, -60000, -90000, -120000]);
  });

  it("computes retention rate as net over income", () => {
    const result = computeProjection(baseInput({ income: 120000, expense: 10000 }));

    expect(result.summary.retentionRate).toBeCloseTo(91.6667, 3);
  });

  it("returns null retention rate when income is zero", () => {
    const result = computeProjection(baseInput({ income: 0, expense: 10000 }));

    expect(result.summary.retentionRate).toBeNull();
  });

  it("supports years period type", () => {
    const result = computeProjection(
      baseInput({ income: 1200000, expense: 0, periodType: "years", periods: 2 })
    );

    expect(result.points.map((p) => p.label)).toEqual(["Now", "1 year", "2 years"]);
    expect(result.summary.finalBalance).toBe(2400000);
  });

  it("respects period limits", () => {
    expect(PERIOD_LIMITS.months).toBe(600);

    const result = computeProjection(baseInput({ periodType: "months", periods: 600 }));
    expect(result.maxPeriods).toBe(600);
    expect(result.points).toHaveLength(601);
  });

  it("sets projected date roughly periods ahead", () => {
    const result = computeProjection(baseInput({ periodType: "months", periods: 10 }));

    const expected = new Date();
    expected.setDate(expected.getDate() + 300);
    const expectedStr = expected.toISOString().split("T")[0];
    expect(result.summary.projectedDate).toBe(expectedStr);
  });
});

describe("periodLabel", () => {
  it("labels step 0 as Now", () => {
    expect(periodLabel("months", 0)).toBe("Now");
    expect(periodLabel("weeks", 0)).toBe("Now");
    expect(periodLabel("years", 0)).toBe("Now");
  });

  it("uses singular for step 1", () => {
    expect(periodLabel("months", 1)).toBe("1 month");
    expect(periodLabel("weeks", 1)).toBe("1 week");
    expect(periodLabel("years", 1)).toBe("1 year");
  });

  it("uses plural for steps > 1", () => {
    expect(periodLabel("months", 10)).toBe("10 months");
    expect(periodLabel("weeks", 3)).toBe("3 weeks");
    expect(periodLabel("years", 2)).toBe("2 years");
  });
});

describe("normalizeSubscriptionToPeriod", () => {
  it("normalizes monthly amounts to months unchanged", () => {
    expect(normalizeSubscriptionToPeriod(10000, "monthly", "months")).toBe(10000);
  });

  it("converts monthly to weeks", () => {
    expect(normalizeSubscriptionToPeriod(12000, "monthly", "weeks")).toBe(2769);
  });

  it("converts monthly to years", () => {
    expect(normalizeSubscriptionToPeriod(10000, "monthly", "years")).toBe(120000);
  });

  it("converts yearly to months", () => {
    expect(normalizeSubscriptionToPeriod(1200000, "yearly", "months")).toBe(100000);
  });

  it("converts weekly to months", () => {
    expect(normalizeSubscriptionToPeriod(1000, "weekly", "months")).toBe(4333);
  });

  it("converts bi_weekly to months", () => {
    expect(normalizeSubscriptionToPeriod(2000, "bi_weekly", "months")).toBe(4333);
  });

  it("converts quarterly to months", () => {
    expect(normalizeSubscriptionToPeriod(30000, "quarterly", "months")).toBe(10000);
  });

  it("converts bi_yearly to months", () => {
    expect(normalizeSubscriptionToPeriod(60000, "bi_yearly", "months")).toBe(10000);
  });

  it("rounds fractional cents", () => {
    expect(normalizeSubscriptionToPeriod(500, "weekly", "weeks")).toBe(500);
    expect(normalizeSubscriptionToPeriod(1, "monthly", "weeks")).toBe(0);
  });

  it("returns 0 for unknown frequencies", () => {
    expect(normalizeSubscriptionToPeriod(10000, "daily", "months")).toBe(0);
  });
});