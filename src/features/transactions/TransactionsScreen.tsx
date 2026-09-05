import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import { formatMoney, formatDate } from "../../services/format";
import type { Transaction, Account, Category, ExportData } from "../../types";
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
  Pencil,
  Eye,
  Upload,
  Download,
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
  const [exportData, setExportData] = useState<ExportData | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<{
    ids: string[];
    count: number;
  } | null>(null);
  const [editingTransaction, setEditingTransaction] = useState<Transaction | null>(null);
  const [viewingTransaction, setViewingTransaction] = useState<Transaction | null>(null);
  const [showExport, setShowExport] = useState(false);
  const PER_PAGE_OPTIONS = [10, 20, 25, 30, 40, 60, 80, 100];

  const load = () => {
    api.getTransactions().then(setTransactions);
    api.getAccounts().then(setAccounts);
    api.getCategories().then(setCategories);
    api.exportData().then(setExportData);
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
                <Upload size={14} style={{ marginRight: 6, verticalAlign: "middle" }} />
                Import CSV
              </button>
              <button className="btn" onClick={() => setShowExport(true)}>
                <Download size={14} style={{ marginRight: 6, verticalAlign: "middle" }} />
                Export Data
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

      {editingTransaction && (
        <EditTransactionForm
          accounts={accounts}
          categories={categories}
          transaction={editingTransaction}
          onDone={() => {
            setEditingTransaction(null);
            load();
          }}
          onCancel={() => setEditingTransaction(null)}
        />
      )}

      {viewingTransaction && (
        <ViewTransactionDialog
          transaction={viewingTransaction}
          account={accountMap[viewingTransaction.account_id]}
          category={viewingTransaction.category_id ? categoryMap[viewingTransaction.category_id] : null}
          onClose={() => setViewingTransaction(null)}
        />
      )}

      {showExport && exportData && (
        <ExportDialog
          data={exportData}
          onDone={() => setShowExport(false)}
          onCancel={() => setShowExport(false)}
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
                  <th>Type</th>
                  <th>Category</th>
                  <th>Account</th>
                  <th style={{ textAlign: "right" }}>Amount</th>
                  <th style={{ width: 90 }}></th>
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
                        <span className={`type-tag ${t.transaction_type}`}>
                          {t.transaction_type}
                        </span>
                      </td>
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
                      <td style={{ textAlign: "right", whiteSpace: "nowrap" }}>
                        <button
                          className="btn"
                          title="View transaction"
                          onClick={() => setViewingTransaction(t)}
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
                          title="Edit transaction"
                          onClick={() => setEditingTransaction(t)}
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
                          title="Delete transaction"
                          onClick={() => setConfirmDelete({ ids: [t.id], count: 1 })}
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

function ExportDialog({
  onDone,
  onCancel,
  data,
}: {
  onDone: () => void;
  onCancel: () => void;
  data: ExportData;
}) {
  const [format, setFormat] = useState<"json" | "csv">("csv");

  const handleExport = async () => {
    if (format === "json") {
      const contents = JSON.stringify(data, null, 2);
      const path = await api.saveFileDialog("chelete-export.json", "json");
      if (path) await api.writeExportFile(path, contents);
    } else {
      const contents = transactionsToCsv(data.transactions, data.accounts, data.categories);
      const path = await api.saveFileDialog("chelete-transactions.csv", "csv");
      if (path) await api.writeExportFile(path, contents);
    }
    onDone();
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Export Data</div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleExport();
          }}
        >
          <div className="form-group">
            <label className="form-label">Format</label>
            <div style={{ display: "flex", gap: 8 }}>
              <button
                type="button"
                className={`btn ${format === "csv" ? "btn-primary" : ""}`}
                onClick={() => setFormat("csv")}
                style={{ flex: 1 }}
              >
                CSV
              </button>
              <button
                type="button"
                className={`btn ${format === "json" ? "btn-primary" : ""}`}
                onClick={() => setFormat("json")}
                style={{ flex: 1 }}
              >
                JSON
              </button>
            </div>
          </div>

          <div className="modal-actions">
            <button type="button" className="btn" onClick={onCancel}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary">
              Export
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function escapeCsvField(value: unknown): string {
  const str = value == null ? "" : String(value);
  if (/[",\n\r]/.test(str)) {
    return `"${str.replace(/"/g, '""')}"`;
  }
  return str;
}

function transactionsToCsv(
  transactions: Transaction[],
  accounts: Account[],
  categories: Category[]
): string {
  const accountMap = Object.fromEntries(accounts.map((a) => [a.id, a.name]));
  const categoryMap = Object.fromEntries(categories.map((c) => [c.id, c.name]));

  const headers = [
    "id",
    "transaction_date",
    "description",
    "merchant",
    "transaction_type",
    "amount",
    "currency",
    "account_name",
    "category_name",
    "notes",
    "created_at",
    "updated_at",
  ];
  const rows = transactions.map((t) => [
    t.id,
    t.transaction_date,
    t.description,
    t.merchant ?? "",
    t.transaction_type,
    (t.amount / 100).toFixed(2),
    t.currency,
    accountMap[t.account_id] ?? t.account_id,
    t.category_id ? categoryMap[t.category_id] ?? t.category_id : "",
    t.notes ?? "",
    t.created_at,
    t.updated_at,
  ]);
  return [headers, ...rows].map((row) => row.map(escapeCsvField).join(",")).join("\n");
}

function ViewTransactionDialog({
  transaction,
  account,
  category,
  onClose,
}: {
  transaction: Transaction;
  account?: Account;
  category?: Category | null;
  onClose: () => void;
}) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Transaction Details</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "8px 16px", fontSize: 14 }}>
          <span style={{ color: "var(--chelete-fg-muted)" }}>Date</span>
          <span>{formatDate(transaction.transaction_date)}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Description</span>
          <span>{transaction.description}</span>

          {transaction.merchant && (
            <>
              <span style={{ color: "var(--chelete-fg-muted)" }}>Merchant</span>
              <span>{transaction.merchant}</span>
            </>
          )}

          <span style={{ color: "var(--chelete-fg-muted)" }}>Type</span>
          <span>
            <span className={`type-tag ${transaction.transaction_type}`}>
              {transaction.transaction_type}
            </span>
          </span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Amount</span>
          <span className={`amount ${transaction.transaction_type}`}>
            {formatMoney(
              transaction.transaction_type === "expense" ? -transaction.amount : transaction.amount
            )}
          </span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Account</span>
          <span>{account?.name || "—"}</span>

          <span style={{ color: "var(--chelete-fg-muted)" }}>Category</span>
          <span>
            {category ? (
              <span className="category-tag">
                {category.icon && (
                  <CategoryIcon name={category.icon} size={12} style={{ marginRight: 4, verticalAlign: "middle" }} />
                )}
                {category.name}
              </span>
            ) : (
              "—"
            )}
          </span>

          {transaction.notes && (
            <>
              <span style={{ color: "var(--chelete-fg-muted)" }}>Notes</span>
              <span>{transaction.notes}</span>
            </>
          )}
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
  return (
    <TransactionForm
      title="Add Transaction"
      accounts={accounts}
      categories={categories}
      onSubmit={async (values) => {
        await api.createTransaction(values);
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function EditTransactionForm({
  accounts,
  categories,
  transaction,
  onDone,
  onCancel,
}: {
  accounts: Account[];
  categories: Category[];
  transaction: Transaction;
  onDone: () => void;
  onCancel: () => void;
}) {
  return (
    <TransactionForm
      title="Edit Transaction"
      accounts={accounts}
      categories={categories}
      initialType={transaction.transaction_type}
      initialAmount={(transaction.amount / 100).toFixed(2)}
      initialDescription={transaction.description}
      initialAccountId={transaction.account_id}
      initialCategoryId={transaction.category_id ?? ""}
      initialDate={transaction.transaction_date}
      onSubmit={async (values) => {
        await api.updateTransaction({
          id: transaction.id,
          account_id: values.account_id,
          category_id: values.category_id || undefined,
          transaction_type: values.transaction_type,
          amount: values.amount,
          currency: "USD",
          description: values.description,
          transaction_date: values.transaction_date,
        });
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function TransactionForm({
  title,
  accounts,
  categories,
  initialType = "expense",
  initialAmount = "",
  initialDescription = "",
  initialAccountId,
  initialCategoryId = "",
  initialDate = new Date().toISOString().split("T")[0],
  onSubmit,
  onCancel,
}: {
  title: string;
  accounts: Account[];
  categories: Category[];
  initialType?: string;
  initialAmount?: string;
  initialDescription?: string;
  initialAccountId?: string;
  initialCategoryId?: string;
  initialDate?: string;
  onSubmit: (values: {
    account_id: string;
    category_id?: string;
    transaction_type: string;
    amount: number;
    currency: string;
    description: string;
    transaction_date: string;
  }) => void | Promise<void>;
  onCancel: () => void;
}) {
  const [type, setType] = useState(initialType);
  const [amount, setAmount] = useState(initialAmount);
  const [description, setDescription] = useState(initialDescription);
  const [accountId, setAccountId] = useState(initialAccountId || accounts[0]?.id || "");
  const [categoryId, setCategoryId] = useState(initialCategoryId);
  const [date, setDate] = useState(initialDate);

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

    await onSubmit({
      account_id: accountId,
      category_id: categoryId || undefined,
      transaction_type: type,
      amount: cents,
      currency: "USD",
      description,
      transaction_date: date,
    });
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">{title}</div>
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
