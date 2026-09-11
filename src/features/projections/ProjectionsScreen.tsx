import { useEffect, useMemo, useState } from "react";
import { api } from "../../services/api";
import {
  computeProjection,
  normalizeSubscriptionToPeriod,
  PERIOD_PLURAL_LABELS,
  PERIOD_LIMITS,
  PERIOD_UNIT_LABELS,
  type ProjectionPeriodType,
} from "../../services/projection";
import { formatBalance, formatMoney, formatDateFull } from "../../services/format";
import type { Subscription } from "../../types";
import { LineChart, TrendingUp, Target, Calendar, Repeat } from "lucide-react";
import {
  ResponsiveContainer,
  LineChart as RechartsLineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ReferenceLine,
} from "recharts";

const PERIOD_OPTIONS: { value: ProjectionPeriodType; label: string }[] = [
  { value: "weeks", label: "Weeks" },
  { value: "months", label: "Months" },
  { value: "years", label: "Years" },
];

type ExpenseSource = "manual" | "subscriptions";

const EXPENSE_SOURCE_OPTIONS: { value: ExpenseSource; label: string }[] = [
  { value: "manual", label: "Manual" },
  { value: "subscriptions", label: "Subscriptions" },
];

export function ProjectionsScreen() {
  const [startingBalance, setStartingBalance] = useState("0");
  const [income, setIncome] = useState("");
  const [expense, setExpense] = useState("");
  const [expenseSource, setExpenseSource] = useState<ExpenseSource>("manual");
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [subscriptionsLoaded, setSubscriptionsLoaded] = useState(false);
  const [periodType, setPeriodType] = useState<ProjectionPeriodType>("months");
  const [periods, setPeriods] = useState("10");

  useEffect(() => {
    if (expenseSource === "subscriptions" && !subscriptionsLoaded) {
      api
        .getSubscriptions()
        .then((subs) => setSubscriptions(subs.filter((s) => s.is_active)))
        .finally(() => setSubscriptionsLoaded(true));
    }
  }, [expenseSource, subscriptionsLoaded]);

  const subscriptionsExpense = useMemo(() => {
    if (expenseSource !== "subscriptions") return 0;
    return subscriptions.reduce(
      (sum, s) => sum + normalizeSubscriptionToPeriod(s.amount, s.frequency, periodType),
      0
    );
  }, [subscriptions, expenseSource, periodType]);

  const parsed = useMemo(() => {
    const startingBalanceCents = parseFloatDollarsToCents(startingBalance);
    const incomeCents = parseFloatDollarsToCents(income);
    const manualExpenseCents = parseFloatDollarsToCents(expense);
    const expenseCents =
      expenseSource === "subscriptions" ? subscriptionsExpense : manualExpenseCents;
    const periodCount = parseInt(periods, 10);
    const valid =
      startingBalanceCents !== null &&
      incomeCents !== null &&
      expenseCents !== null &&
      Number.isFinite(periodCount) &&
      periodCount >= 1 &&
      periodCount <= PERIOD_LIMITS[periodType];
    return { startingBalanceCents, incomeCents, expenseCents, periodCount, valid };
  }, [startingBalance, income, expense, expenseSource, subscriptionsExpense, periodType, periods]);

  const result = useMemo(() => {
    if (!parsed.valid) return null;
    return computeProjection({
      startingBalance: parsed.startingBalanceCents!,
      income: parsed.incomeCents!,
      expense: parsed.expenseCents!,
      periodType,
      periods: parsed.periodCount,
    });
  }, [parsed, periodType]);

  const summary = result?.summary ?? null;
  const growing = (summary?.netPerPeriod ?? 0) > 0;
  const declining = (summary?.netPerPeriod ?? 0) < 0;

  return (
    <div>
      <div className="page-header">
        <div>
          <div className="page-title">Projections</div>
          <div className="page-subtitle">
            Project your balance growth over time
          </div>
        </div>
      </div>

      <div className="projection-layout">
        <div className="card projection-form">
          <div className="form-group">
            <label className="form-label">Starting Balance</label>
            <input
              className="form-input"
              type="number"
              min="0"
              step="0.01"
              placeholder="0.00"
              value={startingBalance}
              onChange={(e) => setStartingBalance(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label className="form-label">Income per Period</label>
            <input
              className="form-input"
              type="number"
              min="0"
              step="0.01"
              placeholder="e.g. 1200.00"
              value={income}
              onChange={(e) => setIncome(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label className="form-label">Expense Source</label>
            <select
              className="form-select"
              value={expenseSource}
              onChange={(e) => setExpenseSource(e.target.value as ExpenseSource)}
            >
              {EXPENSE_SOURCE_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          {expenseSource === "manual" ? (
            <div className="form-group">
              <label className="form-label">Expense per Period</label>
              <input
                className="form-input"
                type="number"
                min="0"
                step="0.01"
                placeholder="e.g. 100.00"
                value={expense}
                onChange={(e) => setExpense(e.target.value)}
              />
            </div>
          ) : (
            <div className="projection-subs-box">
              <div className="projection-subs-header">
                <Repeat size={14} strokeWidth={1.5} />
                <span>
                  {subscriptions.length} active subscription
                  {subscriptions.length === 1 ? "" : "s"}
                </span>
              </div>
              {subscriptions.length > 0 && (
                <div className="projection-subs-list">
                  {subscriptions.map((s) => (
                    <div className="projection-subs-item" key={s.id}>
                      <span className="projection-subs-name">{s.name}</span>
                      <span className="projection-subs-amount">
                        {formatMoney(
                          normalizeSubscriptionToPeriod(s.amount, s.frequency, periodType)
                        )}
                        /{PERIOD_UNIT_LABELS[periodType]}
                      </span>
                    </div>
                  ))}
                </div>
              )}
              <div className="projection-subs-total">
                Total: {formatMoney(subscriptionsExpense)}/{PERIOD_UNIT_LABELS[periodType]}
              </div>
              {subscriptionsLoaded && subscriptions.length === 0 && (
                <div className="projection-subs-empty">
                  No active subscriptions found
                </div>
              )}
            </div>
          )}
          <div className="form-group">
            <label className="form-label">Period Type</label>
            <select
              className="form-select"
              value={periodType}
              onChange={(e) => setPeriodType(e.target.value as ProjectionPeriodType)}
            >
              {PERIOD_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <div className="form-group">
            <label className="form-label">
              Number of {PERIOD_PLURAL_LABELS[periodType]}
            </label>
            <input
              className="form-input"
              type="number"
              min="1"
              max={PERIOD_LIMITS[periodType]}
              step="1"
              placeholder="e.g. 10"
              value={periods}
              onChange={(e) => setPeriods(e.target.value)}
            />
            <div className="projection-hint">
              1 – {PERIOD_LIMITS[periodType]} {PERIOD_PLURAL_LABELS[periodType]}
            </div>
          </div>
        </div>

        <div className="projection-content">
          {!result || !summary ? (
            <div className="projection-empty">
              <LineChart size={32} strokeWidth={1.5} />
              <div>Enter your income, expense and period to see a projection.</div>
            </div>
          ) : (
            <>
              <div className="card-grid">
                <div className="card">
                  <div className="card-header">
                    <div className="card-label">
                      Projected Balance
                    </div>
                    <Target size={32} strokeWidth={1.5} className="card-icon" />
                  </div>
                  <div
                    className={`card-value ${growing ? "positive" : declining ? "negative" : ""}`}
                  >
                    {formatBalance(summary.finalBalance)}
                  </div>
                  <div className="card-change">
                    in {parsed.periodCount} {PERIOD_PLURAL_LABELS[periodType]}
                    {summary.projectedDate
                      ? ` · ${formatDateFull(summary.projectedDate)}`
                      : ""}
                  </div>
                </div>
                <div className="card">
                  <div className="card-header">
                    <div className="card-label">Total Change</div>
                    <TrendingUp size={32} strokeWidth={1.5} className="card-icon" />
                  </div>
                  <div
                    className={`card-value ${summary.totalIncrease >= 0 ? "positive" : "negative"}`}
                  >
                    {formatMoney(summary.totalIncrease)}
                  </div>
                  <div className="card-change">from starting balance</div>
                </div>
                <div className="card">
                  <div className="card-header">
                    <div className="card-label">
                      Net per {capitalize(PERIOD_UNIT_LABELS[periodType])}
                    </div>
                    <Calendar size={32} strokeWidth={1.5} className="card-icon" />
                  </div>
                  <div
                    className={`card-value ${growing ? "positive" : declining ? "negative" : ""}`}
                  >
                    {formatMoney(summary.netPerPeriod)}
                  </div>
                  <div className="card-change">
                    {summary.retentionRate !== null
                      ? `Keeping ${summary.retentionRate.toFixed(1)}% of income`
                      : "No income entered"}
                  </div>
                </div>
              </div>

              <div className="card projection-chart-card">
                <div className="projection-chart-title">Balance Over Time</div>
                <div className="projection-chart-container">
                  <ResponsiveContainer width="100%" height="100%">
                      <RechartsLineChart
                        data={result.points}
                        margin={{ top: 12, right: 24, bottom: 8, left: 8 }}
                      >
                        <CartesianGrid
                          strokeDasharray="3 3"
                          stroke="var(--chelete-border)"
                          vertical={false}
                        />
                        <XAxis
                          dataKey="label"
                          tick={{ fill: "var(--chelete-fg-muted)", fontSize: 11 }}
                          tickLine={false}
                          axisLine={{ stroke: "var(--chelete-border)" }}
                          minTickGap={24}
                        />
                        <YAxis
                          tickFormatter={(v: number) => formatMoneyShortAxis(v)}
                          tick={{ fill: "var(--chelete-fg-muted)", fontSize: 11 }}
                          tickLine={false}
                          axisLine={false}
                          width={72}
                        />
                        <Tooltip
                          formatter={(value) => [formatBalance(Number(value)), "Balance"]}
                          contentStyle={{
                            background: "var(--chelete-surface)",
                            border: "1px solid var(--chelete-border)",
                            borderRadius: 4,
                            fontSize: 12,
                            color: "var(--chelete-fg)",
                          }}
                          labelStyle={{ color: "var(--chelete-fg-muted)" }}
                        />
                        {summary.finalBalance < 0 && (
                          <ReferenceLine y={0} stroke="var(--chelete-fg-muted)" strokeDasharray="4 4" />
                        )}
                        <Line
                          type="monotone"
                          dataKey="balance"
                          stroke={declining ? "#f7768e" : "#7aa2f7"}
                          strokeWidth={2}
                          dot={{ r: 2, fill: declining ? "#f7768e" : "#7aa2f7" }}
                          activeDot={{ r: 4 }}
                        />
                      </RechartsLineChart>
                    </ResponsiveContainer>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function capitalize(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

function parseFloatDollarsToCents(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed === "") return null;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) return null;
  return Math.round(parsed * 100);
}

function formatMoneyShortAxis(amount: number): string {
  const abs = Math.abs(amount);
  const dollars = abs / 100;
  const sign = amount < 0 ? "-" : "";
  if (dollars >= 1_000_000) return `${sign}$${(dollars / 1_000_000).toFixed(1)}M`;
  if (dollars >= 1000) return `${sign}$${(dollars / 1000).toFixed(1)}k`;
  return `${sign}$${dollars.toFixed(0)}`;
}