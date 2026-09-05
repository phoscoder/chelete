import { useEffect, useState, useMemo } from "react";
import { api } from "../../services/api";
import type { Category, Transaction } from "../../types";
import {
  CATEGORY_ICONS,
  CategoryIcon,
} from "../../components/CategoryIcons";
import { Pencil, Trash2 } from "lucide-react";

export function CategoriesScreen() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [editing, setEditing] = useState<Category | null>(null);

  const load = () => {
    api.getCategories().then(setCategories);
    api.getTransactions().then(setTransactions);
  };

  useEffect(() => {
    load();
  }, []);

  const expenseCategories = categories.filter(
    (c) => c.category_type === "expense"
  );
  const incomeCategories = categories.filter(
    (c) => c.category_type === "income"
  );

  const transactionCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    transactions.forEach((t) => {
      if (t.category_id) {
        counts[t.category_id] = (counts[t.category_id] || 0) + 1;
      }
    });
    return counts;
  }, [transactions]);

  const CategoryRow = ({ c }: { c: Category }) => {
    const count = transactionCounts[c.id] || 0;
    return (
      <tr key={c.id}>
        <td>
          {c.icon && (
            <CategoryIcon
              name={c.icon}
              size={14}
              style={{
                marginRight: 6,
                verticalAlign: "middle",
                color: c.color || "var(--chelete-fg-muted)",
              }}
            />
          )}
          {c.name}
        </td>
        <td style={{ textAlign: "center" }}>
          <span
            style={{
              fontSize: 12,
              color: count > 0 ? "var(--chelete-fg-muted)" : "var(--chelete-fg-subtle)",
            }}
          >
            {count} {count === 1 ? "record" : "records"}
          </span>
        </td>
        <td style={{ textAlign: "right" }}>
          <button
            className="btn"
            title="Edit category"
            onClick={() => setEditing(c)}
            style={{
              padding: 4,
              background: "transparent",
              borderColor: "transparent",
              color: "var(--chelete-fg-muted)",
              marginRight: 4,
            }}
          >
            <Pencil size={14} />
          </button>
          <button
            className="btn"
            title="Delete category"
            onClick={async () => {
              await api.deleteCategory(c.id);
              load();
            }}
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
  };

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Categories</div>
        <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
          + Add
        </button>
      </div>

      {showAdd && (
        <AddCategoryForm
          onDone={() => {
            setShowAdd(false);
            load();
          }}
          onCancel={() => setShowAdd(false)}
        />
      )}

      {editing && (
        <EditCategoryForm
          category={editing}
          onDone={() => {
            setEditing(null);
            load();
          }}
          onCancel={() => setEditing(null)}
        />
      )}

      <div style={{ display: "flex", gap: 24 }}>
        <div style={{ flex: 1 }}>
          <div
            className="page-title"
            style={{ fontSize: 14, marginBottom: 12 }}
          >
            Expenses
          </div>
          {expenseCategories.length === 0 ? (
            <div
              style={{
                color: "var(--chelete-fg-muted)",
                fontSize: 12,
                padding: "16px 0",
              }}
            >
              No expense categories
            </div>
          ) : (
            <div className="table-container">
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th style={{ textAlign: "center" }}>Records</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {expenseCategories.map((c) => (
                    <CategoryRow key={c.id} c={c} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        <div style={{ flex: 1 }}>
          <div
            className="page-title"
            style={{ fontSize: 14, marginBottom: 12 }}
          >
            Income
          </div>
          {incomeCategories.length === 0 ? (
            <div
              style={{
                color: "var(--chelete-fg-muted)",
                fontSize: 12,
                padding: "16px 0",
              }}
            >
              No income categories
            </div>
          ) : (
            <div className="table-container">
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th style={{ textAlign: "center" }}>Records</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {incomeCategories.map((c) => (
                    <CategoryRow key={c.id} c={c} />
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function AddCategoryForm({
  onDone,
  onCancel,
}: {
  onDone: () => void;
  onCancel: () => void;
}) {
  return (
    <CategoryForm
      title="Add Category"
      initialName=""
      initialType="expense"
      initialIcon="utensils"
      onSubmit={async (values) => {
        await api.createCategory(values);
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function EditCategoryForm({
  category,
  onDone,
  onCancel,
}: {
  category: Category;
  onDone: () => void;
  onCancel: () => void;
}) {
  return (
    <CategoryForm
      title="Edit Category"
      initialName={category.name}
      initialType={category.category_type}
      initialIcon={category.icon ?? "utensils"}
      onSubmit={async (values) => {
        await api.updateCategory({
          id: category.id,
          name: values.name,
          category_type: values.category_type,
          icon: values.icon,
        });
        onDone();
      }}
      onCancel={onCancel}
    />
  );
}

function CategoryForm({
  title,
  initialName,
  initialType,
  initialIcon,
  onSubmit,
  onCancel,
}: {
  title: string;
  initialName: string;
  initialType: string;
  initialIcon: string;
  onSubmit: (values: { name: string; category_type: string; icon: string }) => void | Promise<void>;
  onCancel: () => void;
}) {
  const [name, setName] = useState(initialName);
  const [type, setType] = useState(initialType);
  const [selectedIcon, setSelectedIcon] = useState(initialIcon);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    await onSubmit({
      name: name.trim(),
      category_type: type,
      icon: selectedIcon,
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
              placeholder="e.g. Groceries"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
            />
          </div>

          <div className="form-group">
            <label className="form-label">Type</label>
            <select
              className="form-select"
              value={type}
              onChange={(e) => setType(e.target.value)}
            >
              <option value="expense">Expense</option>
              <option value="income">Income</option>
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Icon</label>
            <div className="icon-picker">
              {CATEGORY_ICONS.map((ic) => {
                const Icon = ic.icon;
                return (
                  <button
                    key={ic.name}
                    type="button"
                    className={`icon-picker-item ${selectedIcon === ic.name ? "selected" : ""}`}
                    onClick={() => setSelectedIcon(ic.name)}
                    title={ic.label}
                  >
                    <Icon size={16} />
                  </button>
                );
              })}
            </div>
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
