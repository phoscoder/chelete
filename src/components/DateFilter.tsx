import { useState, useRef, useEffect } from "react";
import { ChevronDown, Calendar, Check } from "lucide-react";

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

const FILTERS: { value: DateFilterValue; label: string; section: string }[] = [
  { value: "today", label: "Today", section: "Recent" },
  { value: "yesterday", label: "Yesterday", section: "Recent" },
  { value: "this_week", label: "This Week", section: "Week" },
  { value: "last_week", label: "Last Week", section: "Week" },
  { value: "this_month", label: "This Month", section: "Month" },
  { value: "last_month", label: "Last Month", section: "Month" },
  { value: "this_year", label: "This Year", section: "Year" },
  { value: "last_year", label: "Last Year", section: "Year" },
  { value: "all", label: "All Time", section: "Other" },
  { value: "custom", label: "Custom Range", section: "Other" },
];

const sections = ["Recent", "Week", "Month", "Year", "Other"];

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

  const currentLabel =
    FILTERS.find((f) => f.value === value)?.label || "All Time";

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  return (
    <div className="date-filter-row">
      <div className="date-dropdown" ref={ref}>
        <button
          className="date-dropdown-trigger"
          onClick={() => setOpen(!open)}
        >
          <Calendar size={14} strokeWidth={2} />
          <span className="date-dropdown-label">{currentLabel}</span>
          <ChevronDown
            size={14}
            strokeWidth={2}
            className={`date-dropdown-chevron ${open ? "open" : ""}`}
          />
        </button>

        {open && (
          <div className="date-dropdown-menu">
            {sections.map((section) => {
              const items = FILTERS.filter((f) => f.section === section);
              if (items.length === 0) return null;
              return (
                <div key={section}>
                  <div className="date-dropdown-section">{section}</div>
                  {items.map((f) => (
                    <button
                      key={f.value}
                      className={`date-dropdown-item ${value === f.value ? "active" : ""}`}
                      onClick={() => {
                        onChange(f.value);
                        if (f.value !== "custom") setOpen(false);
                      }}
                    >
                      <span>{f.label}</span>
                      {value === f.value && <Check size={14} strokeWidth={2.5} />}
                    </button>
                  ))}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {value === "custom" && (
        <div className="date-filter-custom-range">
          <input
            type="date"
            className="form-input date-filter-custom-input"
            value={customStart}
            onChange={(e) => onCustomStartChange(e.target.value)}
          />
          <span className="date-filter-custom-separator">to</span>
          <input
            type="date"
            className="form-input date-filter-custom-input"
            value={customEnd}
            onChange={(e) => onCustomEndChange(e.target.value)}
          />
        </div>
      )}
    </div>
  );
}

export function getDateRange(
  filter: DateFilterValue,
  customStart: string,
  customEnd: string
): { start: string; end: string } {
  const now = new Date();
  const today = now.toISOString().split("T")[0];

  const startOfWeek = new Date(now);
  startOfWeek.setDate(now.getDate() - now.getDay());
  const startOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);
  const startOfYear = new Date(now.getFullYear(), 0, 1);

  const endOfLastWeek = new Date(now);
  endOfLastWeek.setDate(now.getDate() - now.getDay() - 1);
  const startOfLastWeek = new Date(endOfLastWeek);
  startOfLastWeek.setDate(endOfLastWeek.getDate() - 6);

  const startOfLastMonth = new Date(now.getFullYear(), now.getMonth() - 1, 1);
  const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);

  const startOfLastYear = new Date(now.getFullYear() - 1, 0, 1);
  const endOfLastYear = new Date(now.getFullYear() - 1, 11, 31);

  const toStr = (d: Date) => d.toISOString().split("T")[0];

  switch (filter) {
    case "today":
      return { start: today, end: today };
    case "yesterday": {
      const yesterday = new Date(now);
      yesterday.setDate(now.getDate() - 1);
      const y = toStr(yesterday);
      return { start: y, end: y };
    }
    case "this_week":
      return { start: toStr(startOfWeek), end: today };
    case "last_week":
      return { start: toStr(startOfLastWeek), end: toStr(endOfLastWeek) };
    case "this_month":
      return { start: toStr(startOfMonth), end: today };
    case "last_month":
      return { start: toStr(startOfLastMonth), end: toStr(endOfLastMonth) };
    case "this_year":
      return { start: toStr(startOfYear), end: today };
    case "last_year":
      return { start: toStr(startOfLastYear), end: toStr(endOfLastYear) };
    case "all":
      return { start: "0000-01-01", end: "9999-12-31" };
    case "custom":
      return {
        start: customStart || "0000-01-01",
        end: customEnd || "9999-12-31",
      };
  }
}

export function isDateInRange(
  dateStr: string,
  start: string,
  end: string
): boolean {
  return dateStr >= start && dateStr <= end;
}
