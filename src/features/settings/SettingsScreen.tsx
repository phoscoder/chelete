import { useTheme } from "../../hooks/useTheme";

export function SettingsScreen() {
  const theme = useTheme();

  return (
    <div>
      <div className="page-header">
        <div className="page-title">Settings</div>
      </div>

      <div style={{ maxWidth: 500 }}>
        <section style={{ marginBottom: 32 }}>
          <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
            Appearance
          </div>
          <div className="card">
            <div className="form-group">
              <label className="form-label">Theme</label>
              <div style={{ fontSize: 13, color: "var(--chelete-fg)" }}>
                Follow Omarchy
              </div>
              <div style={{ fontSize: 11, color: "var(--chelete-fg-muted)", marginTop: 4 }}>
                Detected: {theme?.name || "Loading..."}
              </div>
            </div>
          </div>
        </section>

        <section style={{ marginBottom: 32 }}>
          <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
            Finance
          </div>
          <div className="card">
            <div className="form-group">
              <label className="form-label">Base Currency</label>
              <div style={{ fontSize: 13 }}>USD</div>
            </div>
            <div className="form-group">
              <label className="form-label">Month Start</label>
              <div style={{ fontSize: 13 }}>1st</div>
            </div>
          </div>
        </section>

        <section style={{ marginBottom: 32 }}>
          <div className="page-title" style={{ fontSize: 14, marginBottom: 12 }}>
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
