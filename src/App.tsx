import { useState, useEffect, useCallback } from "react";
import { useTheme } from "./hooks/useTheme";
import { OverviewScreen } from "./features/overview/OverviewScreen";
import { TransactionsScreen } from "./features/transactions/TransactionsScreen";
import { AccountsScreen } from "./features/accounts/AccountsScreen";
import { CategoriesScreen } from "./features/categories/CategoriesScreen";
import { SettingsScreen } from "./features/settings/SettingsScreen";
import { CommandPalette } from "./components/CommandPalette";
import type { View } from "./types";
import {
  LayoutDashboard,
  ArrowLeftRight,
  Wallet,
  Tag,
  Settings,
  Command,
  PanelLeftClose,
  PanelLeftOpen,
  type LucideIcon,
} from "lucide-react";

const NAV_ITEMS: { view: View; label: string; shortcut: string; icon: LucideIcon }[] = [
  { view: "overview", label: "Overview", shortcut: "Ctrl+O", icon: LayoutDashboard },
  { view: "transactions", label: "Transactions", shortcut: "Ctrl+T", icon: ArrowLeftRight },
  { view: "accounts", label: "Accounts", shortcut: "Ctrl+A", icon: Wallet },
  { view: "categories", label: "Categories", shortcut: "Ctrl+C", icon: Tag },
  { view: "settings", label: "Settings", shortcut: "Ctrl+S", icon: Settings },
];

export default function App() {
  const [view, setView] = useState<View>("overview");
  const [commandPalette, setCommandPalette] = useState(false);
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem("sidebar-collapsed") === "true";
  });

  useTheme();

  const toggleCollapsed = () => {
    setCollapsed((prev) => {
      localStorage.setItem("sidebar-collapsed", String(!prev));
      return !prev;
    });
  };

  // Global keyboard shortcuts
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;

      // Command palette: Ctrl+K
      if (ctrl && e.key === "k") {
        e.preventDefault();
        setCommandPalette((o) => !o);
        return;
      }

      // Toggle sidebar: Ctrl+B
      if (ctrl && e.key === "b") {
        e.preventDefault();
        toggleCollapsed();
        return;
      }

      // Escape closes command palette
      if (e.key === "Escape") {
        setCommandPalette(false);
        return;
      }

      // Skip if typing in an input
      const target = e.target as HTMLElement;
      if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") {
        return;
      }

      if (ctrl) {
        const viewMap: Record<string, View> = {
          o: "overview",
          t: "transactions",
          a: "accounts",
          c: "categories",
          s: "settings",
        };
        if (viewMap[e.key]) {
          e.preventDefault();
          setView(viewMap[e.key]);
        }
      }
    },
    []
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  const renderScreen = () => {
    switch (view) {
      case "overview":
        return <OverviewScreen />;
      case "transactions":
        return <TransactionsScreen />;
      case "accounts":
        return <AccountsScreen />;
      case "categories":
        return <CategoriesScreen />;
      case "settings":
        return <SettingsScreen />;
    }
  };

  return (
    <div className="app">
      <aside className={`sidebar ${collapsed ? "collapsed" : ""}`}>
        <div className="sidebar-logo">
          <div className="element-box">
            <span className="element-number">115</span>
            <span className="element-symbol">Ch</span>
          </div>
          {!collapsed && <span className="element-name">Chelete</span>}
        </div>
        <nav>
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.view}
                className={`sidebar-link ${view === item.view ? "active" : ""}`}
                onClick={() => setView(item.view)}
                title={collapsed ? item.label : undefined}
              >
                <Icon size={16} strokeWidth={1.5} />
                {!collapsed && (
                  <>
                    <span>{item.label}</span>
                    <span className="shortcut">{item.shortcut}</span>
                  </>
                )}
              </button>
            );
          })}
        </nav>
        <div className="sidebar-divider" />
        <nav>
          <button
            className="sidebar-link"
            onClick={() => setCommandPalette(true)}
            title={collapsed ? "Command" : undefined}
          >
            <Command size={16} strokeWidth={1.5} />
            {!collapsed && (
              <>
                <span>Command</span>
                <span className="shortcut">Ctrl+K</span>
              </>
            )}
          </button>
        </nav>
        <div className="sidebar-divider" />
        <nav>
          <button className="sidebar-link" onClick={toggleCollapsed}>
            {collapsed ? (
              <PanelLeftOpen size={16} strokeWidth={1.5} />
            ) : (
              <PanelLeftClose size={16} strokeWidth={1.5} />
            )}
            {!collapsed && (
              <>
                <span>Collapse</span>
                <span className="shortcut">Ctrl+B</span>
              </>
            )}
          </button>
        </nav>
      </aside>

      <main className="main-content">{renderScreen()}</main>

      <CommandPalette
        open={commandPalette}
        onClose={() => setCommandPalette(false)}
        onNavigate={(v) => {
          setView(v);
          setCommandPalette(false);
        }}
        onAction={(action) => {
          setCommandPalette(false);
          if (action === "add-transaction") setView("transactions");
          else if (action === "add-account") setView("accounts");
          else if (action === "add-category") setView("categories");
        }}
      />
    </div>
  );
}
