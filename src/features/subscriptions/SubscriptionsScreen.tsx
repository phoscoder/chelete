import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import { formatMoney } from "../../services/format";
import type { Subscription, Category, Account } from "../../types";
import { Pagination, paginate } from "../../components/Pagination";
import { Eye, Pencil, Trash2, CreditCard } from "lucide-react";

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

function getNextPaymentDate(
  startDate: string | null,
  frequency: string
): string | null {
  if (!startDate) return null;

  const start = new Date(startDate + "T00:00:00");
  if (isNaN(start.getTime())) return null;

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const intervals: Record<string, { months?: number; days?: number }> = {
    weekly: { days: 7 },
    bi_weekly: { days: 14 },
    monthly: { months: 1 },
    quarterly: { months: 3 },
    bi_yearly: { months: 6 },
    yearly: { months: 12 },
  };

  const interval = intervals[frequency];
  if (!interval) return startDate;

  const next = new Date(start);
  const maxIterations = 120;
  let iterations = 0;

  if (interval.days) {
    while (next < today && iterations < maxIterations) {
      next.setDate(next.getDate() + interval.days);
      iterations++;
    }
  } else if (interval.months) {
    while (next < today && iterations < maxIterations) {
      const year = next.getFullYear();
      const month = next.getMonth() + interval.months;
      next.setFullYear(year, month, next.getDate());
      iterations++;
    }
  }

  return next.toISOString().split("T")[0];
}

import {
  CategoryIcon,
} from "../../components/CategoryIcons";

export function SubscriptionsScreen() {
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [filterFrequency, setFilterFrequency] = useState<string>("all");
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(25);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [confirmDelete, setConfirmDelete] = useState<{
    ids: string[];
    count: number;
  } | null>(null);
  const [viewing, setViewing] = useState<Subscription | null>(null);
  const [editing, setEditing] = useState<Subscription | null>(null);
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" } | null>(null);
  const PER_PAGE_OPTIONS = [10, 20, 25, 30, 40, 60, 80, 100];

  const load = () => {
    api.getSubscriptions().then(setSubscriptions);
    api.getCategories().then(setCategories);
    api.getAccounts().then(setAccounts);
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    setPage(1);
  }, [filterFrequency, perPage]);

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

  const paginatedSubscriptions = useMemo(
    () => paginate(filteredSubscriptions, page, perPage),
    [filteredSubscriptions, page, perPage]
  );

  const totalAmount = useMemo(() => {
    return filteredSubscriptions.reduce((sum, s) => sum + s.amount, 0);
  }, [filteredSubscriptions]);

  const selectedOnPage = useMemo(() => {
    if (paginatedSubscriptions.length === 0) return false;
    return paginatedSubscriptions.every((s) => selectedIds.has(s.id));
  }, [paginatedSubscriptions, selectedIds]);

  const someSelectedOnPage = useMemo(() => {
    return paginatedSubscriptions.some((s) => selectedIds.has(s.id));
  }, [paginatedSubscriptions, selectedIds]);

  const toggleSelection = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const togglePageSelection = () => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (selectedOnPage) {
        paginatedSubscriptions.forEach((s) => next.delete(s.id));
      } else {
        paginatedSubscriptions.forEach((s) => next.add(s.id));
      }
      return next;
    });
  };

  const handleDelete = async (ids: string[]) => {
    await api.deleteSubscriptions(ids);
    setSelectedIds((prev) => {
      const next = new Set(prev);
      ids.forEach((id) => next.delete(id));
      return next;
    });
    load();
  };

  const showToast = (message: string, type: "success" | "error" = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

  const handlePay = async (s: Subscription) => {
    try {
      const today = new Date().toISOString().split("T")[0];
      await api.createTransaction({
        account_id: s.account_id || accounts[0]?.id,
        category_id: s.category_id || undefined,
        transaction_type: "expense",
        amount: s.amount,
        currency: s.currency,
        description: s.name,
        transaction_date: today,
      });
      showToast(`Payment recorded for ${s.name}`);
      load();
    } catch (err) {
      showToast("Failed to record payment", "error");
    }
  };

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Subscriptions</div>
        <div style={{ display: "flex", gap: 8 }}>
          {selectedIds.size > 0 ? (
            <button
              className="btn btn-danger"
              onClick={() =>
                setConfirmDelete({
                  ids: Array.from(selectedIds),
                  count: selectedIds.size,
                })
              }
            >
              <Trash2 size={14} style={{ marginRight: 6, verticalAlign: "middle" }} />
              Delete {selectedIds.size} selected
            </button>
          ) : (
            <>
              <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
                + Add
              </button>
            </>
          )}
        </div>
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

      {editing && (
        <EditSubscriptionForm
          categories={categories}
          accounts={accounts}
          subscription={editing}
          onDone={() => {
            setEditing(null);
            load();
          }}
          onCancel={() => setEditing(null)}
        />
      )}

      {viewing && (
        <ViewSubscriptionDialog
          subscription={viewing}
          category={viewing.category_id ? categoryMap[viewing.category_id] : null}
          account={viewing.account_id ? accountMap[viewing.account_id] : null}
          onClose={() => setViewing(null)}
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
            <div className="card-label">
              Total ({filterFrequency === "all" ? "All Periods" : FREQUENCY_LABELS[filterFrequency]})
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
        <>
          <div className="table-container">
            <table>
              <thead>
                <tr>
                  <th style={{ width: 32 }}>
                    <input
                      type="checkbox"
                      className="form-checkbox"
                      checked={selectedOnPage}
                      ref={(el) => {
                        if (el) el.indeterminate = someSelectedOnPage && !selectedOnPage;
                      }}
                      onChange={togglePageSelection}
                    />
                  </th>
                  <th>Name</th>
                  <th>Category</th>
                  <th>Account</th>
                  <th>Period</th>
                  <th>Start Date</th>
                  <th>Next Payment</th>
                  <th className="amount">Amount</th>
                  <th style={{ width: 90 }}></th>
                </tr>
              </thead>
              <tbody>
                {paginatedSubscriptions.map((s) => {
                  const category = s.category_id ? categoryMap[s.category_id] : null;
                  const account = s.account_id ? accountMap[s.account_id] : null;
                  const nextPayment = getNextPaymentDate(s.start_date, s.frequency);
                  return (
                    <tr key={s.id}>
                      <td>
                        <input
                          type="checkbox"
                          className="form-checkbox"
                          checked={selectedIds.has(s.id)}
                          onChange={() => toggleSelection(s.id)}
                        />
                      </td>
                      <td>{s.name}</td>
                      <td>
                        {category ? (
                          <span className="category-tag">
                            {category.icon && (
                              <CategoryIcon
                                name={category.icon}
                                size={12}
                                style={{ marginRight: 4, verticalAlign: "middle" }}
                              />
                            )}
                            {category.name}
                          </span>
                        ) : (
                          <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>
                        )}
                      </td>
                      <td>{account?.name || <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>}</td>
                      <td>{FREQUENCY_LABELS[s.frequency] || s.frequency}</td>
                      <td>{s.start_date || <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>}</td>
                      <td>{nextPayment || <span style={{ color: "var(--chelete-fg-subtle)" }}>—</span>}</td>
                      <td className="amount">{formatMoney(s.amount)}</td>
                      <td style={{ textAlign: "right", whiteSpace: "nowrap" }}>
                        <button
                          className="btn"
                          title="View subscription"
                          onClick={() => setViewing(s)}
                          style={{
                            padding: 4,
                            background: "transparent",
                            borderColor: "transparent",
                            color: "var(--chelete-fg-muted)",
                            marginRight: 4,
                            display: "inline-flex",
                            verticalAlign: "middle",
                          }}
                        >
                          <Eye size={14} />
                        </button>
                        <button
                          className="btn"
                          title="Edit subscription"
                          onClick={() => setEditing(s)}
                          style={{
                            padding: 4,
                            background: "transparent",
                            borderColor: "transparent",
                            color: "var(--chelete-fg-muted)",
                            marginRight: 4,
                            display: "inline-flex",
                            verticalAlign: "middle",
                          }}
                        >
                          <Pencil size={14} />
                        </button>
                        <button
                          className="btn"
                          title="Record payment"
                          onClick={() => handlePay(s)}
                          style={{
                            padding: 4,
                            background: "transparent",
                            borderColor: "transparent",
                            color: "var(--chelete-fg-muted)",
                            marginRight: 4,
                            display: "inline-flex",
                            verticalAlign: "middle",
                          }}
                        >
                          <CreditCard size={14} />
                        </button>
                        <button
                          className="btn"
                          title="Delete subscription"
                          onClick={() => setConfirmDelete({ ids: [s.id], count: 1 })}
                          style={{
                            padding: 4,
                            background: "transparent",
                            borderColor: "transparent",
                            color: "var(--chelete-danger)",
                            opacity: 0.75,
                            display: "inline-flex",
                            verticalAlign: "middle",
                          }}
                        >
                          <Trash2 size={14} />
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          <div className="pagination">
            <div className="pagination-per-page">
              <span>Show</span>
              <select
                className="form-select"
                value={perPage}
                onChange={(e) => setPerPage(Number(e.target.value))}
              >
                {PER_PAGE_OPTIONS.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>
              <span>per page</span>
            </div>
            <Pagination
              page={page}
              perPage={perPage}
              total={filteredSubscriptions.length}
              onPageChange={setPage}
            />
          </div>
        </>
      )}

      {confirmDelete && (
        <DeleteConfirmDialog
          count={confirmDelete.count}
          onConfirm={async () => {
            await handleDelete(confirmDelete.ids);
            setConfirmDelete(null);
          }}
          onCancel={() => setConfirmDelete(null)}
        />
      )}

      {toast && (
        <div
          style={{
            position: "fixed",
            bottom: 20,
            right: 20,
            padding: "12px 16px",
            borderRadius: "var(--chelete-radius)",
            background:
              toast.type === "success"
                ? "var(--chelete-success, #22c55e)"
                : "var(--chelete-danger)",
            color: "#fff",
            boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
            fontSize: 14,
            zIndex: 1000,
          }}
        >
          {toast.message}
        </div>
      )}
    </div>
  );
}

function DeleteConfirmDialog({
  count,
  onConfirm,
  onCancel,
}: {
  count: number;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">
          Delete {count} subscription{count === 1 ? "" : "s"}?
        </div>
        <p style={{ marginBottom: 20, fontSize: 14 }}>
          This will permanently remove the selected subscription{count === 1 ? "" : "s"}. This action
          cannot be undone.
        </p>
        <div className="modal-actions">
          <button className="btn" onClick={onCancel}>
            Cancel
          </button>
          <button className="btn btn-danger" onClick={onConfirm}>
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}

function ViewSubscriptionDialog({
  subscription,
  category,
  account,
  onClose,
}: {
  subscription: Subscription;
  category?: Category | null;
  account?: Account | null;
  onClose: () => void;
}) {
  const nextPayment = getNextPaymentDate(subscription.start_date, subscription.frequency);
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Subscription Details</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "8px 16px", fontSize: 14 }}>
          <span style={{ color: "var(--chelete-fg-muted)" }}>Name</span>
          <span>{subscription.name}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Amount</span>
          <span className="amount">{formatMoney(subscription.amount)}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Period</span>
          <span>{FREQUENCY_LABELS[subscription.frequency] || subscription.frequency}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Start Date</span>
          <span>{subscription.start_date || "—"}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Next Payment</span>
          <span>{nextPayment || "—"}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Account</span>
          <span>{account?.name || "—"}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Category</span>
          <span>
            {category ? (
              <span className="category-tag">
                {category.icon && (
                  <CategoryIcon
                    name={category.icon}
                    size={12}
                    style={{ marginRight: 4, verticalAlign: "middle" }}
                  />
                )}
                {category.name}
              </span>
            ) : (
              "—"
            )}
          </span>
        </div>
        <div className="modal-actions" style={{ marginTop: 16 }}>
          <button className="btn btn-primary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
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
  return (
    <SubscriptionForm
      title="Add Subscription"
      categories={categories}
      accounts={accounts}
      onSubmit={async (values) => {
        await api.createSubscription(values);
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function EditSubscriptionForm({
  categories,
  accounts,
  subscription,
  onDone,
  onCancel,
}: {
  categories: Category[];
  accounts: Account[];
  subscription: Subscription;
  onDone: () => void;
  onCancel: () => void;
}) {
  return (
    <SubscriptionForm
      title="Edit Subscription"
      categories={categories}
      accounts={accounts}
      initialName={subscription.name}
      initialAmount={(subscription.amount / 100).toFixed(2)}
      initialFrequency={subscription.frequency}
      initialCategoryId={subscription.category_id ?? ""}
      initialAccountId={subscription.account_id ?? ""}
      initialStartDate={subscription.start_date ?? ""}
      onSubmit={async (values) => {
        await api.updateSubscription({
          id: subscription.id,
          name: values.name,
          amount: values.amount,
          currency: "USD",
          frequency: values.frequency,
          category_id: values.category_id || undefined,
          account_id: values.account_id || undefined,
          start_date: values.start_date || undefined,
        });
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function SubscriptionForm({
  title,
  categories,
  accounts,
  initialName = "",
  initialAmount = "",
  initialFrequency = "monthly",
  initialCategoryId = "",
  initialAccountId = "",
  initialStartDate = "",
  onSubmit,
  onCancel,
}: {
  title: string;
  categories: Category[];
  accounts: Account[];
  initialName?: string;
  initialAmount?: string;
  initialFrequency?: string;
  initialCategoryId?: string;
  initialAccountId?: string;
  initialStartDate?: string;
  onSubmit: (values: {
    name: string;
    amount: number;
    currency: string;
    frequency: string;
    category_id?: string;
    account_id?: string;
    start_date?: string;
  }) => void | Promise<void>;
  onCancel: () => void;
}) {
  const [name, setName] = useState(initialName);
  const [amount, setAmount] = useState(initialAmount);
  const [frequency, setFrequency] = useState(initialFrequency);
  const [categoryId, setCategoryId] = useState(initialCategoryId);
  const [accountId, setAccountId] = useState(initialAccountId);
  const [startDate, setStartDate] = useState(initialStartDate);

  const expenseCategories = categories.filter((c) => c.category_type === "expense");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !amount) return;

    const cents = Math.round(parseFloat(amount) * 100);
    await onSubmit({
      name: name.trim(),
      amount: cents,
      currency: "USD",
      frequency,
      category_id: categoryId || undefined,
      account_id: accountId || undefined,
      start_date: startDate || undefined,
    });
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">{title}</div>
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
              {title === "Add Subscription" ? "Add" : "Save"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
