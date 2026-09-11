import { useTheme } from "../../hooks/useTheme";
import { useEffect, useState } from "react";

const APP_VERSION = "0.6.0";

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
            About
          </div>
          <div className="card about-card">
            <div className="about-header">
              <div className="element-box about-logo">
                <span className="element-number">115</span>
                <span className="element-symbol">Ch</span>
              </div>
              <div>
                <div className="about-name">Chelete</div>
                <div className="about-version">Version {APP_VERSION}</div>
              </div>
            </div>
            <div className="about-desc">
              A fast, keyboard-driven personal finance tracker for Linux.
              Manage accounts, track transactions, and project your balance
              over time — without touching the mouse. Built for Omarchy with
              live theme integration.
            </div>
            <div className="settings-divider" />
            <div className="settings-row">
              <div>
                <div className="settings-label">Author</div>
                <div className="settings-desc">The Architect &amp; QA</div>
              </div>
              <div className="settings-value">Victor Phos</div>
            </div>
            <div className="settings-divider" />
            <div className="settings-row">
              <div>
                <div className="settings-label">License</div>
              </div>
              <div className="settings-value">MIT</div>
            </div>
          </div>
        </section>

      </div>
    </div>
  );
}
