import { useEffect, useState } from "react";
import { api } from "../../services/api";
import { formatBalance, accountTypeLabel } from "../../services/format";
import type { Account } from "../../types";

export function AccountsScreen() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showAdd, setShowAdd] = useState(false);

  const load = () => {
    api.getAccounts().then(setAccounts);
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Accounts</div>
        <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
          + Add
        </button>
      </div>

      {showAdd && (
        <AddAccountForm
          onDone={() => {
            setShowAdd(false);
            load();
          }}
          onCancel={() => setShowAdd(false)}
        />
      )}

      {accounts.length === 0 ? (
        <div style={{ textAlign: "center", padding: "48px 0", color: "var(--chelete-fg-muted)" }}>
          No accounts yet. Add one to get started.
        </div>
      ) : (
        <div className="card-grid">
          {accounts.map((a) => (
            <div className="card" key={a.id}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "start" }}>
                <div>
                  <div className="card-label">{accountTypeLabel(a.account_type)}</div>
                  <div style={{ fontSize: 14, fontWeight: 500, marginBottom: 8 }}>
                    {a.name}
                  </div>
                </div>
              </div>
              <div className="card-value">{formatBalance(a.balance)}</div>
              <div style={{ marginTop: 12, display: "flex", gap: 6 }}>
                <button
                  className="btn"
                  style={{ fontSize: 11, padding: "4px 8px" }}
                  onClick={async () => {
                    await api.deleteAccount(a.id);
                    load();
                  }}
                >
                  Delete
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function AddAccountForm({
  onDone,
  onCancel,
}: {
  onDone: () => void;
  onCancel: () => void;
}) {
  const [name, setName] = useState("");
  const [type, setType] = useState("cash");
  const [balance, setBalance] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    const cents = Math.round(parseFloat(balance || "0") * 100);
    await api.createAccount({
      name: name.trim(),
      account_type: type,
      currency: "USD",
      balance: cents,
    });
    onDone();
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Add Account</div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="form-label">Name</label>
            <input
              className="form-input"
              type="text"
              placeholder="e.g. Main Bank"
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
              <option value="cash">Cash</option>
              <option value="bank">Bank</option>
              <option value="savings">Savings</option>
              <option value="credit_card">Credit Card</option>
              <option value="mobile_money">Mobile Money</option>
              <option value="investment">Investment</option>
              <option value="other">Other</option>
            </select>
          </div>

          <div className="form-group">
            <label className="form-label">Starting Balance</label>
            <input
              className="form-input"
              type="number"
              step="0.01"
              placeholder="0.00"
              value={balance}
              onChange={(e) => setBalance(e.target.value)}
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
