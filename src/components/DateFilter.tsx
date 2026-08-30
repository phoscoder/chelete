import { useState, useRef, useEffect } from "react";
import { ChevronDown, Calendar, Check, ArrowRight } from "lucide-react";

export type DateFilterValue =
  | "today"
  | "yesterday"
  | "this_week"
  | "last_week"
  | "this_month"
  | "last_month"
  | "this_year"
  | "last_year"
  | "all"
  | "custom";

interface DateFilterProps {
  value: DateFilterValue;
  onChange: (value: DateFilterValue) => void;
  customStart: string;
  customEnd: string;
  onCustomStartChange: (v: string) => void;
  onCustomEndChange: (v: string) => void;
}

const FILTERS: { value: DateFilterValue; label: string }[] = [
  { value: "today", label: "Today" },
  { value: "yesterday", label: "Yesterday" },
  { value: "this_week", label: "This Week" },
  { value: "last_week", label: "Last Week" },
  { value: "this_month", label: "This Month" },
  { value: "last_month", label: "Last Month" },
  { value: "this_year", label: "This Year" },
  { value: "last_year", label: "Last Year" },
  { value: "all", label: "All Time" },
  { value: "custom", label: "Custom Range" },
];

export function DateFilter({
  value,
  onChange,
  customStart,
  customEnd,
  onCustomStartChange,
  onCustomEndChange,
}: DateFilterProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const label = FILTERS.find((f) => f.value === value)?.label || "All Time";

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  return (
    <div className="df-row">
      <div className="df" ref={ref}>
        <button className="df-btn" onClick={() => setOpen(!open)}>
          <Calendar size={13} strokeWidth={2} />
          <span>{label}</span>
          <ChevronDown size={12} strokeWidth={2.5} className={open ? "df-chevron-open" : ""} />
        </button>

        {open && (
          <div className="df-menu">
            {FILTERS.map((f) => (
              <button
                key={f.value}
                className={`df-opt ${value === f.value ? "df-opt-active" : ""}`}
                onClick={() => { onChange(f.value); if (f.value !== "custom") setOpen(false); }}
              >
                {f.label}
                {value === f.value && <Check size={13} strokeWidth={3} />}
              </button>
            ))}
          </div>
        )}
      </div>

      {value === "custom" && (
        <div className="df-range">
          <input type="date" className="df-date" value={customStart} onChange={(e) => onCustomStartChange(e.target.value)} />
          <ArrowRight size={11} strokeWidth={2} className="df-arrow" />
          <input type="date" className="df-date" value={customEnd} onChange={(e) => onCustomEndChange(e.target.value)} />
        </div>
      )}
    </div>
  );
}

export function getDateRange(filter: DateFilterValue, customStart: string, customEnd: string): { start: string; end: string } {
  const now = new Date();
  const today = now.toISOString().split("T")[0];
  const toStr = (d: Date) => d.toISOString().split("T")[0];

  const startOfWeek = new Date(now); startOfWeek.setDate(now.getDate() - now.getDay());
  const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
  const startOfYear = new Date(now.getFullYear(), 0, 1);

  const endOfLastWeek = new Date(now); endOfLastWeek.setDate(now.getDate() - now.getDay() - 1);
  const startOfLastWeek = new Date(endOfLastWeek); startOfLastWeek.setDate(endOfLastWeek.getDate() - 6);

  const startOfLastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);
  const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);

  const startOfLastYear = new Date(now.getFullYear() - 1, 0, 1);
  const endOfLastYear = new Date(now.getFullYear() - 1, 11, 31);

  switch (filter) {
    case "today": return { start: today, end: today };
    case "yesterday": { const y = toStr(new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1)); return { start: y, end: y }; }
    case "this_week": return { start: toStr(startOfWeek), end: today };
    case "last_week": return { start: toStr(startOfLastWeek), end: toStr(endOfLastWeek) };
    case "this_month": return { start: toStr(startOfMonth), end: today };
    case "last_month": return { start: toStr(startOfLastMonth), end: toStr(endOfLastMonth) };
    case "this_year": return { start: toStr(startOfYear), end: today };
    case "last_year": return { start: toStr(startOfLastYear), end: toStr(endOfLastYear) };
    case "all": return { start: "0000-01-01", end: "9999-12-31" };
    case "custom": return { start: customStart || "0000-01-01", end: customEnd || "9999-12-31" };
  }
}

export function isDateInRange(dateStr: string, start: string, end: string): boolean {
  return dateStr >= start && dateStr <= end;
}
