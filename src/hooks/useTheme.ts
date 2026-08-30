import { useEffect, useState } from "react";
import { api } from "../services/api";
import type { OmarchyTheme } from "../types";

export function useTheme() {
  const [theme, setTheme] = useState<OmarchyTheme | null>(null);

  useEffect(() => {
    let lastThemeJson = "";

    const poll = () => {
      api.getOmarchyTheme().then((t) => {
        const json = JSON.stringify(t);
        if (json !== lastThemeJson) {
          lastThemeJson = json;
          setTheme(t);
          applyTheme(t);
        }
      });
    };

    poll();
    const interval = setInterval(poll, 2000);
    return () => clearInterval(interval);
  }, []);

  return theme;
}

function applyTheme(theme: OmarchyTheme) {
  const root = document.documentElement;
  const c = theme.colors;

  root.style.setProperty("--chelete-bg", c.background || "#1a1b26");
  root.style.setProperty("--chelete-bg-secondary", darken(c.background, 0.1));
  root.style.setProperty("--chelete-surface", c.surface || darken(c.background, -0.08));
  root.style.setProperty("--chelete-surface-hover", c.surface_hover || darken(c.background, -0.15));
  root.style.setProperty("--chelete-fg", theme.foreground || "#c0caf5");
  root.style.setProperty("--chelete-fg-muted", c.foreground_muted || "#737aa2");
  root.style.setProperty("--chelete-fg-subtle", c.foreground_subtle || "#565f89");
  root.style.setProperty("--chelete-accent", theme.accent || "#7aa2f7");
  root.style.setProperty("--chelete-success", c.success || c.green || "#9ece6a");
  root.style.setProperty("--chelete-danger", c.danger || c.red || "#f7768e");
  root.style.setProperty("--chelete-warning", c.warning || c.yellow || "#e0af68");
  root.style.setProperty("--chelete-info", c.info || c.cyan || "#7dcfff");

  const border = c.border || (isLight(c.background) ? darken(c.background, -0.15) : darken(c.background, 0.2));
  root.style.setProperty("--chelete-border", border);
}

function darken(hex: string, amount: number): string {
  const h = hex.replace("#", "");
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);

  const factor = amount > 0 ? 1 + amount : 1 + amount;
  const nr = Math.min(255, Math.max(0, Math.round(r * factor)));
  const ng = Math.min(255, Math.max(0, Math.round(g * factor)));
  const nb = Math.min(255, Math.max(0, Math.round(b * factor)));

  return `#${nr.toString(16).padStart(2, "0")}${ng.toString(16).padStart(2, "0")}${nb.toString(16).padStart(2, "0")}`;
}

function isLight(hex: string): boolean {
  const h = hex.replace("#", "");
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 > 128;
}
