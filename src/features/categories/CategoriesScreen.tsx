import { useEffect, useState } from "react";
import { api } from "../../services/api";
import type { Category } from "../../types";
import {
  CATEGORY_ICONS,
  CategoryIcon,
} from "../../components/CategoryIcons";

export function CategoriesScreen() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [showAdd, setShowAdd] = useState(false);

  const load = () => {
    api.getCategories().then(setCategories);
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
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {expenseCategories.map((c) => (
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
                      <td style={{ textAlign: "right" }}>
                        <button
                          className="btn"
                          style={{ fontSize: 11, padding: "2px 6px" }}
                          onClick={async () => {
                            await api.deleteCategory(c.id);
                            load();
                          }}
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
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
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {incomeCategories.map((c) => (
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
                      <td style={{ textAlign: "right" }}>
                        <button
                          className="btn"
                          style={{ fontSize: 11, padding: "2px 6px" }}
                          onClick={async () => {
                            await api.deleteCategory(c.id);
                            load();
                          }}
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
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
  const [name, setName] = useState("");
  const [type, setType] = useState("expense");
  const [selectedIcon, setSelectedIcon] = useState("utensils");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    await api.createCategory({
      name: name.trim(),
      category_type: type,
      icon: selectedIcon,
    });
    onDone();
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Add Category</div>
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
