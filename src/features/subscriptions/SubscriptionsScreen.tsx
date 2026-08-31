import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import { formatMoney } from "../../services/format";
import type { Subscription, Category, Account } from "../../types";

const FREQUENCIES = [
  { value: "weekly", label: "Weekly" },
  { value: "bi_weekly", label: "Bi-weekly" },
  { value: "monthly", label: "Monthly" },
  { value: "quarterly", label: "Quarterly" },
  { value: "bi_yearly", label: "Bi-yearly" },
  { value: "yearly", label: "Yearly" },
];

const FREQUENCY_LABELS: Record<string, string> = Object.fromEntries(
  FREQUENCIES.map((f) => [f.value, f.label])
);

export function SubscriptionsScreen() {
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [filterFrequency, setFilterFrequency] = useState<string>("all");

  const load = () => {
    api.getSubscriptions().then(setSubscriptions);
    api.getCategories().then(setCategories);
    api.getAccounts().then(setAccounts);
  };

  useEffect(() => {
    load();
  }, []);

  const categoryMap = useMemo(
    () => Object.fromEntries(categories.map((c) => [c.id, c])),
    [categories]
  );

  const accountMap = useMemo(
    () => Object.fromEntries(accounts.map((a) => [a.id, a])),
    [accounts]
  );

  const filteredSubscriptions = useMemo(() => {
    if (filterFrequency === "all") return subscriptions;
    return subscriptions.filter((s) => s.frequency === filterFrequency);
  }, [subscriptions, filterFrequency]);

  const totalAmount = useMemo(() => {
    return filteredSubscriptions.reduce((sum, s) => sum + s.amount, 0);
  }, [filteredSubscriptions]);

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Subscriptions</div>
        <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
          + Add
        </button>
      </div>

      {showAdd && (
        <AddSubscriptionForm
          categories={categories}
          accounts={accounts}
          onDone={() => {
            setShowAdd(false);
            load();
          }}
          onCancel={() => setShowAdd(false)}
        />
      )}

      <div className="overview-filters">
        <select
          className="form-select filter-select"
          value={filterFrequency}
          onChange={(e) => setFilterFrequency(e.target.value)}
        >
          <option value="all">All Periods</option>
          {FREQUENCIES.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>

      <div className="card-grid" style={{ marginBottom: 16 }}>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Total ({filterFrequency === "all" ? "All Periods" : FREQUENCY_LABELS[filterFrequency]})
            </div>
          </div>
          <div className="card-value">{formatMoney(totalAmount)}</div>
        </div>
        <div className="card">
          <div className="card-header">
            <div className="card-label">Subscriptions</div>
          </div>
          <div className="card-value">{filteredSubscriptions.length}</div>
        </div>
      </div>

      {filteredSubscriptions.length === 0 ? (
        <div
          style={{
            textAlign: "center",
            padding: "48px 0",
            color: "var(--chelete-fg-muted)",
          }}
        >
          No subscriptions found.
        </div>
      ) : (
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>Category</th>
                <th>Account</th>
                <th>Period</th>
                <th>Start Date</th>
                <th className="amount">Amount</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {filteredSubscriptions.map((s) => {
                const category = s.category_id ? categoryMap[s.category_id] : null;
                const account = s.account_id ? accountMap[s.account_id] : null;
                return (
                  <tr key={s.id}>
                    <td>{s.name}</td>
                    <td>
                      {category ? (
                        <span className="category-tag">{category.name}</span>
                      ) : (
                        <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>
                      )}
                    </td>
                    <td>{account?.name || <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>}</td>
                    <td>{FREQUENCY_LABELS[s.frequency] || s.frequency}</td>
                    <td>{s.start_date || <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>}</td>
                    <td className="amount">{formatMoney(s.amount)}</td>
                    <td>
                      <button
                        className="btn"
                        style={{ fontSize: 11, padding: "4px 8px" }}
                        onClick={async () => {
                          await api.deleteSubscription(s.id);
                          load();
                        }}
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function AddSubscriptionForm({
  categories,
  accounts,
  onDone,
  onCancel,
}: {
  categories: Category[];
  accounts: Account[];
  onDone: () => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState("");
  const [amount, setAmount] = useState("");
  const [frequency, setFrequency] = useState("monthly");
  const [categoryId, setCategoryId] = useState("");
  const [accountId, setAccountId] = useState("");
  const [startDate, setStartDate] = useState("");

  const expenseCategories = categories.filter((c) => c.category_type === "expense");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !amount) return;

    const cents = Math.round(parseFloat(amount) * 100);
    await api.createSubscription({
      name: name.trim(),
      amount: cents,
      currency: "USD",
      frequency,
      category_id: categoryId || undefined,
      account_id: accountId || undefined,
      start_date: startDate || undefined,
    });
    onDone();
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Add Subscription</div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="form-label">Name</label>
            <input
              className="form-input"
              type="text"
              placeholder="e.g. Netflix"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
            />
          </div>

          <div className="form-group">
            <label className="form-label">Amount</label>
            <input
              className="form-input"
              type="number"
              step="0.01"
              placeholder="0.00"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </div>

          <div className="form-group">
            <label className="form-label">Period</label>
            <select
              className="form-select"
              value={frequency}
              onChange={(e) => setFrequency(e.target.value)}
            >
              {FREQUENCIES.map((f) => (
                <option key={f.value} value={f.value}>
                  {f.label}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Category</label>
            <select
              className="form-select"
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
            >
              <option value="">None</option>
              {expenseCategories.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Account</label>
            <select
              className="form-select"
              value={accountId}
              onChange={(e) => setAccountId(e.target.value)}
            >
              <option value="">None</option>
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Start Date</label>
            <input
              className="form-input"
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
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
