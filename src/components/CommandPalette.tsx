import { useEffect, useState, useRef } from "react";
import type { View } from "../types";

interface Command {
  id: string;
  label: string;
  shortcut?: string;
  action: () => void;
}

export function CommandPalette({
  open,
  onClose,
  onNavigate,
  onAction,
}: {
  open: boolean;
  onClose: () => void;
  onNavigate: (view: View) => void;
  onAction: (action: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const commands: Command[] = [
    { id: "overview", label: "Go to Overview", shortcut: "Ctrl+O", action: () => onNavigate("overview") },
    { id: "transactions", label: "Go to Transactions", shortcut: "Ctrl+T", action: () => onNavigate("transactions") },
    { id: "accounts", label: "Go to Accounts", shortcut: "Ctrl+A", action: () => onNavigate("accounts") },
    { id: "categories", label: "Go to Categories", shortcut: "Ctrl+C", action: () => onNavigate("categories") },
    { id: "subscriptions", label: "Go to Subscriptions", shortcut: "Ctrl+U", action: () => onNavigate("subscriptions") },
    { id: "settings", label: "Go to Settings", shortcut: "Ctrl+S", action: () => onNavigate("settings") },
    { id: "add-transaction", label: "Add Transaction", action: () => onAction("add-transaction") },
    { id: "import-transactions", label: "Import Transactions (CSV)", action: () => onAction("import-transactions") },
    { id: "add-account", label: "Add Account", action: () => onAction("add-account") },
    { id: "add-category", label: "Add Category", action: () => onAction("add-category") },
    { id: "add-subscription", label: "Add Subscription", action: () => onAction("add-subscription") },
  ];

  const filtered = commands.filter((c) =>
    c.label.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelected(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    setSelected(0);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filtered[selected]) {
        filtered[selected].action();
        onClose();
      }
    } else if (e.key === "Escape") {
      onClose();
    }
  };

  if (!open) return null;

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          type="text"
          placeholder="Type a command..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
        />
        <div className="command-palette-results">
          {filtered.map((cmd, i) => (
            <div
              key={cmd.id}
              className={`command-palette-item ${i === selected ? "selected" : ""}`}
              onClick={() => {
                cmd.action();
                onClose();
              }}
              onMouseEnter={() => setSelected(i)}
            >
              <span>{cmd.label}</span>
              {cmd.shortcut && <span className="shortcut">{cmd.shortcut}</span>}
            </div>
          ))}
          {filtered.length === 0 && (
            <div style={{ padding: "16px", textAlign: "center", color: "var(--chelete-fg-muted)", fontSize: 12 }}>
              No matching commands
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
