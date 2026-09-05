import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import { formatMoney, formatDate } from "../../services/format";
import type { Transaction, Account, Category } from "../../types";
import { CsvImportDialog } from "./CsvImportDialog";
import {
  DateFilter,
  getDateRange,
  isDateInRange,
  type DateFilterValue,
} from "../../components/DateFilter";
import { Pagination, paginate } from "../../components/Pagination";
import {
  ShoppingCart,
  Car,
  Briefcase,
  Film,
  ShoppingBag,
  Zap,
  Heart,
  BookOpen,
  Coffee,
  Plane,
  Home,
  Utensils,
  Wifi,
  Smartphone,
  Music,
  Gamepad2,
  Gift,
  Stethoscope,
  GraduationCap,
  Building2,
  Trash2,
  type LucideIcon,
} from "lucide-react";

const CATEGORY_ICONS: { name: string; icon: LucideIcon }[] = [
  { name: "utensils", icon: Utensils },
  { name: "car", icon: Car },
  { name: "briefcase", icon: Briefcase },
  { name: "film", icon: Film },
  { name: "shopping-bag", icon: ShoppingBag },
  { name: "zap", icon: Zap },
  { name: "heart", icon: Heart },
  { name: "book-open", icon: BookOpen },
  { name: "coffee", icon: Coffee },
  { name: "plane", icon: Plane },
  { name: "home", icon: Home },
  { name: "shopping-cart", icon: ShoppingCart },
  { name: "wifi", icon: Wifi },
  { name: "smartphone", icon: Smartphone },
  { name: "music", icon: Music },
  { name: "gamepad-2", icon: Gamepad2 },
  { name: "gift", icon: Gift },
  { name: "stethoscope", icon: Stethoscope },
  { name: "graduation-cap", icon: GraduationCap },
  { name: "building-2", icon: Building2 },
];

function CategoryIcon({
  name,
  size = 14,
  style,
}: {
  name: string;
  size?: number;
  style?: React.CSSProperties;
}) {
  const iconEntry = CATEGORY_ICONS.find((i) => i.name === name);
  if (iconEntry) {
    const Icon = iconEntry.icon;
    return <Icon size={size} style={style} />;
  }
  return null;
}

export function TransactionsScreen() {
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [showImport, setShowImport] = useState(false);
  const [dateFilter, setDateFilter] = useState<DateFilterValue>("all");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [transactionPage, setTransactionPage] = useState(1);
  const [transactionsPerPage, setTransactionsPerPage] = useState(25);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [confirmDelete, setConfirmDelete] = useState<{
    ids: string[];
    count: number;
  } | null>(null);
  const PER_PAGE_OPTIONS = [10, 20, 25, 30, 40, 60, 80, 100];

  const load = () => {
    api.getTransactions().then(setTransactions);
    api.getAccounts().then(setAccounts);
    api.getCategories().then(setCategories);
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    const onOpenImport = () => setShowImport(true);
    window.addEventListener("chelete-open-import", onOpenImport);
    return () => window.removeEventListener("chelete-open-import", onOpenImport);
  }, []);

  const accountMap = Object.fromEntries(accounts.map((a) => [a.id, a]));
  const categoryMap = Object.fromEntries(categories.map((c) => [c.id, c]));

  const { start, end } = getDateRange(dateFilter, customStart, customEnd);

  const filteredTransactions = useMemo(() => {
    return transactions.filter((t) => isDateInRange(t.transaction_date, start, end));
  }, [transactions, start, end]);

  useEffect(() => {
    setTransactionPage(1);
  }, [dateFilter, customStart, customEnd, transactionsPerPage]);

  const paginatedTransactions = useMemo(
    () => paginate(filteredTransactions, transactionPage, transactionsPerPage),
    [filteredTransactions, transactionPage, transactionsPerPage]
  );

  const selectedOnPage = useMemo(() => {
    if (paginatedTransactions.length === 0) return false;
    return paginatedTransactions.every((t) => selectedIds.has(t.id));
  }, [paginatedTransactions, selectedIds]);

  const someSelectedOnPage = useMemo(() => {
    return paginatedTransactions.some((t) => selectedIds.has(t.id));
  }, [paginatedTransactions, selectedIds]);

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
        paginatedTransactions.forEach((t) => next.delete(t.id));
      } else {
        paginatedTransactions.forEach((t) => next.add(t.id));
      }
      return next;
    });
  };

  const handleDelete = async (ids: string[]) => {
    await api.deleteTransactions(ids);
    setSelectedIds((prev) => {
      const next = new Set(prev);
      ids.forEach((id) => next.delete(id));
      return next;
    });
    load();
  };

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Transactions</div>
        <div style={{ display: "flex", gap: 8 }}>
          {selectedIds.size > 0 ? (
            <button
              className="btn btn-danger"
              onClick={() =>
                setConfirmDelete({ ids: Array.from(selectedIds), count: selectedIds.size })
              }
            >
              Delete {selectedIds.size} selected
            </button>
          ) : (
            <>
              <button className="btn" onClick={() => setShowImport(true)}>
                Import CSV
              </button>
              <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
                + Add
              </button>
            </>
          )}
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

      {showImport && (
        <CsvImportDialog
          accounts={accounts}
          categories={categories}
          onDone={() => {
            setShowImport(false);
            load();
          }}
          onCancel={() => setShowImport(false)}
        />
      )}

      {filteredTransactions.length === 0 ? (
        <div
          style={{
            textAlign: "center",
            padding: "48px 0",
            color: "var(--chelete-fg-muted)",
            marginTop: 16,
          }}
        >
          {transactions.length === 0
            ? "No transactions yet. Press "
            : "No transactions match the selected filters."}
          {transactions.length === 0 && (
            <>
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
              to add one.
            </>
          )}
        </div>
      ) : (
        <>
          <div className="table-container" style={{ marginTop: 16 }}>
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
                  <th>Date</th>
                  <th>Description</th>
                  <th>Category</th>
                  <th>Account</th>
                  <th style={{ textAlign: "right" }}>Amount</th>
                  <th style={{ width: 40 }}></th>
                </tr>
              </thead>
              <tbody>
                {paginatedTransactions.map((t) => {
                  const cat = t.category_id ? categoryMap[t.category_id] : null;
                  return (
                    <tr key={t.id}>
                      <td>
                        <input
                          type="checkbox"
                          className="form-checkbox"
                          checked={selectedIds.has(t.id)}
                          onChange={() => toggleSelection(t.id)}
                        />
                      </td>
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
                          <span style={{ color: "var(--chelete-fg-subtle)" }}>
                            —
                          </span>
                        )}
                      </td>
                      <td>{accountMap[t.account_id]?.name || "—"}</td>
                      <td className={`amount ${t.transaction_type}`}>
                        {formatMoney(
                          t.transaction_type === "expense" ? -t.amount : t.amount
                        )}
                      </td>
                      <td>
                        <button
                          className="btn"
                          title="Delete transaction"
                          onClick={() => setConfirmDelete({ ids: [t.id], count: 1 })}
                          style={{
                            padding: 4,
                            background: "transparent",
                            borderColor: "transparent",
                            color: "var(--chelete-danger)",
                            opacity: 0.75,
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
                value={transactionsPerPage}
                onChange={(e) => setTransactionsPerPage(Number(e.target.value))}
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
              page={transactionPage}
              perPage={transactionsPerPage}
              total={filteredTransactions.length}
              onPageChange={setTransactionPage}
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
        <div className="modal-title">Delete {count} transaction{count === 1 ? "" : "s"}?</div>
        <p style={{ marginBottom: 20, fontSize: 14 }}>
          This will permanently remove the selected transaction{count === 1 ? "" : "s"} and reverse
          any account balance changes. This action cannot be undone.
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

  const expenseCategories = categories.filter(
    (c) => c.category_type === "expense"
  );
  const incomeCategories = categories.filter(
    (c) => c.category_type === "income"
  );
  const filteredCategories =
    type === "expense" ? expenseCategories : incomeCategories;

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
