import { useEffect, useState } from "react";
import { api } from "../../services/api";
import { formatBalance, accountTypeLabel } from "../../services/format";
import type { Account } from "../../types";
import { ArrowRightLeft } from "lucide-react";

export function AccountsScreen() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [showTransfer, setShowTransfer] = useState(false);
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" } | null>(null);

  const showNotification = (message: string, type: "success" | "error" = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  };

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
        <div style={{ display: "flex", gap: 8 }}>
          <button className="btn" onClick={() => setShowTransfer(true)}>
            <ArrowRightLeft size={14} style={{ marginRight: 6, verticalAlign: "middle" }} />
            Transfer
          </button>
          <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
            + Add
          </button>
        </div>
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

      {showTransfer && (
        <TransferDialog
          accounts={accounts}
          onDone={() => {
            setShowTransfer(false);
            load();
          }}
          onCancel={() => setShowTransfer(false)}
          showNotification={showNotification}
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

function TransferDialog({
  accounts,
  onDone,
  onCancel,
  showNotification,
}: {
  accounts: Account[];
  onDone: () => void;
  onCancel: () => void;
  showNotification: (message: string, type?: "success" | "error") => void;
}) {
  const [fromAccountId, setFromAccountId] = useState(accounts[0]?.id || "");
  const [toAccountId, setToAccountId] = useState(
    accounts[1]?.id || accounts[0]?.id || ""
  );
  const [amount, setAmount] = useState("");
  const [notes, setNotes] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const cents = Math.round(parseFloat(amount) * 100);
    if (isNaN(cents) || cents <= 0 || fromAccountId === toAccountId) return;

    try {
      await api.transfer({
        from_account_id: fromAccountId,
        to_account_id: toAccountId,
        amount: cents,
        currency: "USD",
        notes: notes || undefined,
      });
      showNotification(
        `Transferred $${amount} to ${accounts.find((a) => a.id === toAccountId)?.name}`
      );
      onDone();
    } catch (err: any) {
      showNotification(
        `Transfer failed: ${err?.message || String(err)}`,
        "error"
      );
    }
  };

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-title">Transfer Money</div>
        <form onSubmit={handleSubmit}>
          <div style={{ display: "flex", gap: 8 }}>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">From</label>
              <select
                className="form-select"
                value={fromAccountId}
                onChange={(e) => setFromAccountId(e.target.value)}
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group" style={{ flex: 1 }}>
              <label className="form-label">To</label>
              <select
                className="form-select"
                value={toAccountId}
                onChange={(e) => setToAccountId(e.target.value)}
              >
                {accounts.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
            </div>
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
            <label className="form-label">Notes</label>
            <input
              className="form-input"
              type="text"
              placeholder="e.g. Monthly savings"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
            />
          </div>

          <div className="modal-actions">
            <button type="button" className="btn" onClick={onCancel}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={fromAccountId === toAccountId || !amount}
            >
              Transfer
            </button>
          </div>
        </form>
      </div>
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
