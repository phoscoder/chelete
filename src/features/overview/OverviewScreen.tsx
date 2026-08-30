import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import {
  formatBalance,
  formatMoney,
  formatDate,
  currentMonth,
} from "../../services/format";
import type { Overview } from "../../types";
import {
  Wallet,
  TrendingUp,
  TrendingDown,
  Activity,
  CreditCard,
  Tag,
} from "lucide-react";

export function OverviewScreen() {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [filterType, setFilterType] = useState<"all" | "income" | "expense">("all");
  const [filterAccount, setFilterAccount] = useState<string>("all");

  useEffect(() => {
    api.getOverview().then(setOverview);
  }, []);

  const accounts = useMemo(() => overview?.accounts ?? [], [overview]);

  const filteredTransactions = useMemo(() => {
    if (!overview) return [];
    let txns = overview.recent_transactions;
    if (filterType !== "all") {
      txns = txns.filter((t) => t.transaction_type === filterType);
    }
    if (filterAccount !== "all") {
      txns = txns.filter((t) => t.account_id === filterAccount);
    }
    return txns;
  }, [overview, filterType, filterAccount]);

  if (!overview) {
    return <div className="page-header"><span className="page-title">Loading...</span></div>;
  }

  const net = overview.total_income - overview.total_expenses;
  const savingsRate =
    overview.total_income > 0
      ? Math.round((net / overview.total_income) * 100)
      : 0;

  return (
    <div>
      <div className="page-header">
        <div>
          <div className="page-title">Overview</div>
          <div className="page-subtitle">{currentMonth()}</div>
        </div>
      </div>

      <div className="card-grid">
        <div className="card">
          <div className="card-header">
            <div className="card-label">Total Balance</div>
            <Wallet size={16} strokeWidth={1.5} className="card-icon" />
          </div>
          <div className="card-value">{formatBalance(overview.total_balance)}</div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Income</div>
            <TrendingUp size={16} strokeWidth={1.5} className="card-icon positive" />
          </div>
          <div className="card-value positive">
            {formatMoney(overview.total_income)}
          </div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Expenses</div>
            <TrendingDown size={16} strokeWidth={1.5} className="card-icon negative" />
          </div>
          <div className="card-value negative">
            {formatMoney(-overview.total_expenses)}
          </div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Net</div>
            <Activity size={16} strokeWidth={1.5} className="card-icon" />
          </div>
          <div className={`card-value ${net >= 0 ? "positive" : "negative"}`}>
            {formatMoney(net)}
          </div>
          <div className="card-change">Savings rate: {savingsRate}%</div>
        </div>
      </div>

      {accounts.length > 0 && (
        <>
          <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
            Accounts
          </div>
          <div className="card-grid">
            {accounts.map((a) => (
              <div className="card" key={a.id}>
                <div className="card-header">
                  <div className="card-label">{a.name}</div>
                  <CreditCard size={14} strokeWidth={1.5} className="card-icon" />
                </div>
                <div className="card-value">{formatBalance(a.balance)}</div>
              </div>
            ))}
          </div>
        </>
      )}

      <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
        Recent Transactions
      </div>
      <div className="overview-filters">
        <select
          className="form-select filter-select"
          value={filterType}
          onChange={(e) => setFilterType(e.target.value as typeof filterType)}
        >
          <option value="all">All Types</option>
          <option value="income">Income</option>
          <option value="expense">Expense</option>
        </select>
        <select
          className="form-select filter-select"
          value={filterAccount}
          onChange={(e) => setFilterAccount(e.target.value)}
        >
          <option value="all">All Accounts</option>
          {accounts.map((a) => (
            <option key={a.id} value={a.id}>{a.name}</option>
          ))}
        </select>
      </div>

      {filteredTransactions.length > 0 ? (
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Date</th>
                <th>Description</th>
                <th>Amount</th>
              </tr>
            </thead>
            <tbody>
              {filteredTransactions.map((t) => (
                <tr key={t.id}>
                  <td style={{ whiteSpace: "nowrap" }}>
                    {formatDate(t.transaction_date)}
                  </td>
                  <td>{t.description}</td>
                  <td className={`amount ${t.transaction_type}`}>
                    {formatMoney(
                      t.transaction_type === "expense" ? -t.amount : t.amount
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div style={{ color: "var(--chelete-fg-muted)", fontSize: 13, padding: "16px 0" }}>
          No transactions match the selected filters.
        </div>
      )}

      {overview.category_spending.length > 0 && (
        <>
          <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
            <Tag size={14} strokeWidth={1.5} style={{ marginRight: 6, verticalAlign: "middle" }} />
            Spending by Category
          </div>
          {overview.category_spending.map((cs) => {
            const pct =
              overview.total_expenses > 0
                ? Math.round((cs.spent / overview.total_expenses) * 100)
                : 0;
            return (
              <div className="budget-row" key={cs.category_id}>
                <div className="budget-header">
                  <span className="budget-name">{cs.category_name}</span>
                  <span className="budget-amounts">
                    {formatMoneyShort(cs.spent)} ({pct}%)
                  </span>
                </div>
                <div className="progress-bar">
                  <div
                    className="progress-fill ok"
                    style={{ width: `${Math.min(pct, 100)}%` }}
                  />
                </div>
              </div>
            );
          })}
        </>
      )}

      {filteredTransactions.length === 0 &&
        overview.accounts.length === 0 && (
          <div
            style={{
              textAlign: "center",
              padding: "48px 0",
              color: "var(--chelete-fg-muted)",
            }}
          >
            <div style={{ fontSize: 14, marginBottom: 8 }}>
              Welcome to Chelete
            </div>
            <div style={{ fontSize: 12 }}>
              Press <kbd style={{
                padding: "2px 6px",
                background: "var(--chelete-surface)",
                border: "1px solid var(--chelete-border)",
                borderRadius: 3,
                fontFamily: "var(--chelete-mono)",
                fontSize: 11,
              }}>Ctrl+K</kbd> to open the command palette, or{" "}
              <kbd style={{
                padding: "2px 6px",
                background: "var(--chelete-surface)",
                border: "1px solid var(--chelete-border)",
                borderRadius: 3,
                fontFamily: "var(--chelete-mono)",
                fontSize: 11,
              }}>a</kbd> to add your first transaction.
            </div>
          </div>
        )}
    </div>
  );
}

function formatMoneyShort(amount: number): string {
  const abs = Math.abs(amount);
  const dollars = abs / 100;
  if (dollars >= 1000) {
    return `$${(dollars / 1000).toFixed(1)}k`;
  }
  return `$${dollars.toFixed(2)}`;
}
