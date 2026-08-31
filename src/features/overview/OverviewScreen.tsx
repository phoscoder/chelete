import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import {
  formatBalance,
  formatMoney,
  formatDate,
  currentMonth,
} from "../../services/format";
import type { Overview, Transaction, Account, Category } from "../../types";
import {
  Wallet,
  TrendingUp,
  TrendingDown,
  Activity,
  CreditCard,
} from "lucide-react";
import {
  DateFilter,
  getDateRange,
  isDateInRange,
  type DateFilterValue,
} from "../../components/DateFilter";
import { DonutChart } from "../../components/DonutChart";
import { Pagination, paginate } from "../../components/Pagination";
import { CategoryIcon } from "../../components/CategoryIcons";

export function OverviewScreen() {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [allTransactions, setAllTransactions] = useState<Transaction[]>([]);
  const [filterType, setFilterType] = useState<"all" | "income" | "expense">("all");
  const [filterAccount, setFilterAccount] = useState<string>("all");
  const [dateFilter, setDateFilter] = useState<DateFilterValue>("all");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [transactionPage, setTransactionPage] = useState(1);
  const TRANSACTIONS_PER_PAGE = 10;

  useEffect(() => {
    api.getOverview().then(setOverview);
    api.getAccounts().then(setAccounts);
    api.getCategories().then(setCategories);
    api.getTransactions().then(setAllTransactions);
  }, []);

  const categoryMap = useMemo(
    () => Object.fromEntries(categories.map((c) => [c.id, c])),
    [categories]
  );

  const { start, end } = getDateRange(dateFilter, customStart, customEnd);

  const filteredTransactions = useMemo(() => {
    let txns = allTransactions.filter((t) => isDateInRange(t.transaction_date, start, end));
    if (filterType !== "all") {
      txns = txns.filter((t) => t.transaction_type === filterType);
    }
    if (filterAccount !== "all") {
      txns = txns.filter((t) => t.account_id === filterAccount);
    }
    return txns;
  }, [allTransactions, start, end, filterType, filterAccount]);

  useEffect(() => {
    setTransactionPage(1);
  }, [filterType, filterAccount, dateFilter, customStart, customEnd]);

  const filteredOverview = useMemo(() => {
    if (!overview) return null;
    const dateTxns = allTransactions.filter((t) =>
      isDateInRange(t.transaction_date, start, end)
    );
    const total_income = dateTxns
      .filter((t) => t.transaction_type === "income")
      .reduce((sum, t) => sum + t.amount, 0);
    const total_expenses = dateTxns
      .filter((t) => t.transaction_type === "expense")
      .reduce((sum, t) => sum + t.amount, 0);

    const spentByCategory: Record<string, number> = {};
    const earnedByCategory: Record<string, number> = {};
    for (const t of dateTxns) {
      if (t.category_id) {
        if (t.transaction_type === "expense") {
          spentByCategory[t.category_id] =
            (spentByCategory[t.category_id] || 0) + t.amount;
        } else if (t.transaction_type === "income") {
          earnedByCategory[t.category_id] =
            (earnedByCategory[t.category_id] || 0) + t.amount;
        }
      }
    }

    const category_spending = Object.entries(spentByCategory)
      .map(([category_id, spent]) => ({
        category_id,
        category_name: categoryMap[category_id]?.name || "Unknown",
        spent,
        budget_limit: null as number | null,
        icon: categoryMap[category_id]?.icon || null,
      }))
      .sort((a, b) => b.spent - a.spent);

    const income_by_category = Object.entries(earnedByCategory)
      .map(([category_id, value]) => ({
        label: categoryMap[category_id]?.name || "Unknown",
        value,
        color: categoryMap[category_id]?.color || null,
      }))
      .sort((a, b) => b.value - a.value);

    const expenses_by_category = Object.entries(spentByCategory)
      .map(([category_id, value]) => ({
        label: categoryMap[category_id]?.name || "Unknown",
        value,
        color: categoryMap[category_id]?.color || null,
      }))
      .sort((a, b) => b.value - a.value);

    return {
      ...overview,
      total_income,
      total_expenses,
      category_spending,
      income_by_category,
      expenses_by_category,
    };
  }, [overview, allTransactions, start, end, categoryMap]);

  if (!overview || !filteredOverview) {
    return (
      <div className="page-header">
        <span className="page-title">Loading...</span>
      </div>
    );
  }

  const net = filteredOverview.total_income - filteredOverview.total_expenses;
  const savingsRate =
    filteredOverview.total_income > 0
      ? Math.round((net / filteredOverview.total_income) * 100)
      : 0;

  return (
    <div>
      <div className="page-header">
        <div>
          <div className="page-title">Overview</div>
          <div className="page-subtitle">{currentMonth()}</div>
        </div>
      </div>

      <DateFilter
        value={dateFilter}
        onChange={setDateFilter}
        customStart={customStart}
        customEnd={customEnd}
        onCustomStartChange={setCustomStart}
        onCustomEndChange={setCustomEnd}
      />

      <div className="card-grid" style={{ marginTop: 16 }}>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Total Balance</div>
            <Wallet size={32} strokeWidth={1.5} className="card-icon" />
          </div>
          <div className="card-value">
            {formatBalance(overview.total_balance)}
          </div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Income</div>
            <TrendingUp size={32} strokeWidth={1.5} className="card-icon positive" />
          </div>
          <div className="card-value positive">
            {formatMoney(filteredOverview.total_income)}
          </div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Expenses</div>
            <TrendingDown size={32} strokeWidth={1.5} className="card-icon negative" />
          </div>
          <div className="card-value negative">
            {formatMoney(-filteredOverview.total_expenses)}
          </div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Net</div>
            <Activity size={32} strokeWidth={1.5} className="card-icon" />
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
                  <CreditCard size={28} strokeWidth={1.5} className="card-icon" />
                </div>
                <div className="card-value">{formatBalance(a.balance)}</div>
              </div>
            ))}
          </div>
        </>
      )}

      <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
        Breakdown
      </div>
      <div className="donut-chart-grid">
        <DonutChart
          title="Income"
          data={filteredOverview.income_by_category}
          emptyMessage="No income for this period"
        />
        <DonutChart
          title="Expenses"
          data={filteredOverview.expenses_by_category}
          emptyMessage="No expenses for this period"
        />
      </div>

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
            <option key={a.id} value={a.id}>
              {a.name}
            </option>
          ))}
        </select>
      </div>

      {filteredTransactions.length > 0 ? (
        <>
          <div className="table-container">
            <table>
              <thead>
                <tr>
                  <th>Date</th>
                  <th>Description</th>
                  <th>Category</th>
                  <th>Amount</th>
                </tr>
              </thead>
              <tbody>
                {paginate(filteredTransactions, transactionPage, TRANSACTIONS_PER_PAGE).map((t) => {
                const cat = t.category_id ? categoryMap[t.category_id] : null;
                return (
                  <tr key={t.id}>
                    <td style={{ whiteSpace: "nowrap" }}>
                      {formatDate(t.transaction_date)}
                    </td>
                    <td>{t.description}</td>
                    <td>
                      {cat ? (
                        <span className="category-tag">
                          {cat.icon && (
                            <CategoryIcon
                              name={cat.icon}
                              size={12}
                              style={{ marginRight: 4, verticalAlign: "middle" }}
                            />
                          )}
                          {cat.name}
                        </span>
                      ) : (
                        <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>
                      )}
                    </td>
                    <td className={`amount ${t.transaction_type}`}>
                      {formatMoney(
                        t.transaction_type === "expense" ? -t.amount : t.amount
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
          <Pagination
            page={transactionPage}
            perPage={TRANSACTIONS_PER_PAGE}
            total={filteredTransactions.length}
            onPageChange={setTransactionPage}
          />
      </>
      ) : (
        <div
          style={{
            color: "var(--chelete-fg-muted)",
            fontSize: 13,
            padding: "16px 0",
          }}
        >
          No transactions match the selected filters.
        </div>
      )}

      {filteredTransactions.length === 0 && overview.accounts.length === 0 && (
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
            Press{" "}
            <kbd
              style={{
                padding: "2px 6px",
                background: "var(--chelete-surface)",
                border: "1px solid var(--chelete-border)",
                borderRadius: 3,
                fontFamily: "var(--chelete-mono)",
                fontSize: 11,
              }}
            >
              Ctrl+K
            </kbd>{" "}
            to open the command palette, or{" "}
            <kbd
              style={{
                padding: "2px 6px",
                background: "var(--chelete-surface)",
                border: "1px solid var(--chelete-border)",
                borderRadius: 3,
                fontFamily: "var(--chelete-mono)",
                fontSize: 11,
              }}
            >
              a
            </kbd>{" "}
            to add your first transaction.
          </div>
        </div>
      )}
    </div>
  );
}


