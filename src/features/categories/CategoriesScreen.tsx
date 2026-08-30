import { useEffect, useState } from "react";
import { api } from "../../services/api";
import type { Category } from "../../types";
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
  PiggyBank,
  CreditCard,
  Wallet,
  type LucideIcon,
} from "lucide-react";

const CATEGORY_ICONS: { name: string; icon: LucideIcon; label: string }[] = [
  { name: "utensils", icon: Utensils, label: "Food" },
  { name: "car", icon: Car, label: "Car" },
  { name: "briefcase", icon: Briefcase, label: "Work" },
  { name: "film", icon: Film, label: "Film" },
  { name: "shopping-bag", icon: ShoppingBag, label: "Shopping" },
  { name: "zap", icon: Zap, label: "Bills" },
  { name: "heart", icon: Heart, label: "Health" },
  { name: "book-open", icon: BookOpen, label: "Education" },
  { name: "coffee", icon: Coffee, label: "Coffee" },
  { name: "plane", icon: Plane, label: "Travel" },
  { name: "home", icon: Home, label: "Home" },
  { name: "shopping-cart", icon: ShoppingCart, label: "Cart" },
  { name: "wifi", icon: Wifi, label: "Internet" },
  { name: "smartphone", icon: Smartphone, label: "Phone" },
  { name: "music", icon: Music, label: "Music" },
  { name: "gamepad-2", icon: Gamepad2, label: "Gaming" },
  { name: "gift", icon: Gift, label: "Gift" },
  { name: "stethoscope", icon: Stethoscope, label: "Medical" },
  { name: "graduation-cap", icon: GraduationCap, label: "School" },
  { name: "building-2", icon: Building2, label: "Office" },
  { name: "piggy-bank", icon: PiggyBank, label: "Savings" },
  { name: "credit-card", icon: CreditCard, label: "Credit" },
  { name: "wallet", icon: Wallet, label: "Wallet" },
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
