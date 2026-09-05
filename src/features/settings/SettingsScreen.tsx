import { useTheme } from "../../hooks/useTheme";
import { useEffect, useState } from "react";
import { api } from "../../services/api";
import type { Transaction, Account, Category } from "../../types";

export function SettingsScreen() {
  const theme = useTheme();
  const [rounded, setRounded] = useState(() => {
    return localStorage.getItem("chelete-radius") !== "square";
  });

  useEffect(() => {
    const radius = rounded ? "4px" : "0px";
    document.documentElement.style.setProperty("--chelete-radius", radius);
    localStorage.setItem("chelete-radius", rounded ? "rounded" : "square");
  }, [rounded]);

  const handleExportJson = async () => {
    const data = await api.exportData();
    const contents = JSON.stringify(data, null, 2);
    const path = await api.saveFileDialog("chelete-export.json", "json");
    if (path) {
      await api.writeExportFile(path, contents);
    }
  };

  const handleExportCsv = async () => {
    const data = await api.exportData();
    const contents = transactionsToCsv(data.transactions, data.accounts, data.categories);
    const path = await api.saveFileDialog("chelete-transactions.csv", "csv");
    if (path) {
      await api.writeExportFile(path, contents);
    }
  };

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Settings</div>
      </div>

      <div style={{ maxWidth: 500 }}>
        <section style={{ marginBottom: 32 }}>
          <div
            className="page-title"
            style={{ fontSize: 14, marginBottom: 12 }}
          >
            Appearance
          </div>
          <div className="card">
            <div className="settings-row">
              <div>
                <div className="settings-label">Theme</div>
                <div className="settings-desc">
                  Follows Omarchy system theme
                </div>
              </div>
              <div className="settings-value">
                {theme?.name || "Loading..."}
              </div>
            </div>

            <div className="settings-divider" />

            <div className="settings-row">
              <div>
                <div className="settings-label">Corners</div>
                <div className="settings-desc">
                  {rounded ? "Rounded" : "Square"} corners
                </div>
              </div>
              <button
                className={`toggle-switch ${rounded ? "on" : ""}`}
                onClick={() => setRounded(!rounded)}
                aria-label="Toggle corner style"
              >
                <span className="toggle-knob" />
              </button>
            </div>
          </div>
        </section>

        <section style={{ marginBottom: 32 }}>
          <div
            className="page-title"
            style={{ fontSize: 14, marginBottom: 12 }}
          >
            Finance
          </div>
          <div className="card">
            <div className="settings-row">
              <div>
                <div className="settings-label">Base Currency</div>
              </div>
              <div className="settings-value">USD</div>
            </div>
            <div className="settings-divider" />
            <div className="settings-row">
              <div>
                <div className="settings-label">Month Start</div>
              </div>
              <div className="settings-value">1st</div>
            </div>
          </div>
        </section>

        <section style={{ marginBottom: 32 }}>
          <div
            className="page-title"
            style={{ fontSize: 14, marginBottom: 12 }}
          >
            Data
          </div>
          <div className="card">
            <div style={{ display: "flex", gap: 8 }}>
              <button className="btn" onClick={() => handleExportJson()}>
                Export JSON
              </button>
              <button className="btn" onClick={() => handleExportCsv()}>
                Export CSV
              </button>
            </div>
          </div>
        </section>
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
    "amount_cents",
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
    t.amount,
    t.currency,
    accountMap[t.account_id] ?? t.account_id,
    t.category_id ? categoryMap[t.category_id] ?? t.category_id : "",
    t.notes ?? "",
    t.created_at,
    t.updated_at,
  ]);
  return [headers, ...rows].map((row) => row.map(escapeCsvField).join(",")).join("\n");
}
