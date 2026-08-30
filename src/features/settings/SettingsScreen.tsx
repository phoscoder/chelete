import { useTheme } from "../../hooks/useTheme";
import { useEffect, useState } from "react";

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
            Data
          </div>
          <div className="card">
            <div style={{ display: "flex", gap: 8 }}>
              <button className="btn">Export JSON</button>
              <button className="btn">Export CSV</button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
