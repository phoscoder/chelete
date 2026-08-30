import { useEffect, useState } from "react";
import { api } from "../../services/api";
import { formatMoney, formatDate } from "../../services/format";
import type { Transaction, Account, Category } from "../../types";

export function TransactionsScreen() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [showAdd, setShowAdd] = useState(false);

  const load = () => {
    api.getTransactions().then(setTransactions);
    api.getAccounts().then(setAccounts);
    api.getCategories().then(setCategories);
  };

  useEffect(() => {
    load();
  }, []);

  const accountMap = Object.fromEntries(accounts.map((a) => [a.id, a]));
  const categoryMap = Object.fromEntries(categories.map((c) => [c.id, c]));

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Transactions</div>
        <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
          + Add
        </button>
      </div>

      {showAdd && (
        <AddTransactionForm
          accounts={accounts}
          categories={categories}
          onDone={() => {
            setShowAdd(false);
            load();
          }}
          onCancel={() => setShowAdd(false)}
        />
      )}

      {transactions.length === 0 ? (
        <div style={{ textAlign: "center", padding: "48px 0", color: "var(--chelete-fg-muted)" }}>
          No transactions yet. Press{' '}
          <kbd style={{
            padding: "2px 6px",
            background: "var(--chelete-surface)",
            border: "1px solid var(--chelete-border)",
            borderRadius: 3,
            fontFamily: "var(--chelete-mono)",
            fontSize: 11,
          }}>a</kbd>{' '}
          to add one.
        </div>
      ) : (
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Date</th>
                <th>Description</th>
                <th>Category</th>
                <th>Account</th>
                <th style={{ textAlign: "right" }}>Amount</th>
              </tr>
            </thead>
            <tbody>
              {transactions.map((t) => (
                <tr key={t.id}>
                  <td style={{ whiteSpace: "nowrap" }}>{formatDate(t.transaction_date)}</td>
                  <td>{t.description}</td>
                  <td>
                    {t.category_id && categoryMap[t.category_id] ? (
                      <span className="category-tag">{categoryMap[t.category_id].name}</span>
                    ) : (
                      <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>
                    )}
                  </td>
                  <td>{accountMap[t.account_id]?.name || "—"}</td>
                  <td className={`amount ${t.transaction_type}`}>
                    {formatMoney(t.transaction_type === "expense" ? -t.amount : t.amount)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function AddTransactionForm({
  accounts,
  categories,
  onDone,
  onCancel,
}: {
  accounts: Account[];
  categories: Category[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [type, setType] = useState("expense");
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [accountId, setAccountId] = useState(accounts[0]?.id || "");
  const [categoryId, setCategoryId] = useState("");
  const [date, setDate] = useState(new Date().toISOString().split("T")[0]);

  const expenseCategories = categories.filter((c) => c.category_type === "expense");
  const incomeCategories = categories.filter((c) => c.category_type === "income");
  const filteredCategories = type === "expense" ? expenseCategories : incomeCategories;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const cents = Math.round(parseFloat(amount) * 100);
    if (isNaN(cents) || cents <= 0) return;

    await api.createTransaction({
      account_id: accountId,
      category_id: categoryId || undefined,
      transaction_type: type,
      amount: cents,
      currency: "USD",
      description,
      transaction_date: date,
    });
    onDone();
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Add Transaction</div>
        <form onSubmit={handleSubmit}>
          <div style={{ display: "flex", gap: 8, marginBottom: 14 }}>
            <button
              type="button"
              className={`btn ${type === "expense" ? "btn-danger" : ""}`}
              onClick={() => setType("expense")}
              style={{ flex: 1 }}
            >
              Expense
            </button>
            <button
              type="button"
              className={`btn ${type === "income" ? "btn-primary" : ""}`}
              onClick={() => setType("income")}
              style={{ flex: 1 }}
            >
              Income
            </button>
          </div>

          <div className="form-group">
            <label className="form-label">Amount</label>
            <input
              className="form-input"
              type="number"
              step="0.01"
              min="0"
              placeholder="0.00"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              autoFocus
            />
          </div>

          <div className="form-group">
            <label className="form-label">Description</label>
            <input
              className="form-input"
              type="text"
              placeholder="e.g. Groceries"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>

          <div style={{ display: "flex", gap: 8 }}>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Account</label>
              <select
                className="form-select"
                value={accountId}
                onChange={(e) => setAccountId(e.target.value)}
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">Category</label>
              <select
                className="form-select"
                value={categoryId}
                onChange={(e) => setCategoryId(e.target.value)}
              >
                <option value="">None</option>
                {filteredCategories.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="form-group">
            <label className="form-label">Date</label>
            <input
              className="form-input"
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
            />
          </div>

          <div className="modal-actions">
            <button type="button" className="btn" onClick={onCancel}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              Add
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
