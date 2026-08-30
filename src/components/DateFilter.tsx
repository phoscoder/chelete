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

const FILTERS: { value: DateFilterValue; label: string; section: string }[] = [
  { value: "today", label: "Today", section: "Recent" },
  { value: "yesterday", label: "Yesterday", section: "Recent" },
  { value: "this_week", label: "This Week", section: "Week" },
  { value: "last_week", label: "Last Week", section: "Week" },
  { value: "this_month", label: "This Month", section: "Month" },
  { value: "last_month", label: "Last Month", section: "Month" },
  { value: "this_year", label: "This Year", section: "Year" },
  { value: "last_year", label: "Last Year", section: "Year" },
  { value: "all", label: "All Time", section: "" },
  { value: "custom", label: "Custom Range", section: "" },
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

  const grouped = FILTERS.reduce(
    (acc, f) => {
      if (f.section) {
        if (!acc.find((g) => g.section === f.section)) {
          acc.push({ section: f.section, items: [] });
        }
        acc.find((g) => g.section === f.section)!.items.push(f);
      } else {
        if (!acc.find((g) => g.section === "")) {
          acc.push({ section: "", items: [] });
        }
        acc.find((g) => g.section === "")!.items.push(f);
      }
      return acc;
    },
    [] as { section: string; items: typeof FILTERS }[]
  );

  return (
    <div className="df-row">
      <div className="df" ref={ref}>
        <button
          className={`df-trigger ${open ? "df-trigger--open" : ""}`}
          onClick={() => setOpen(!open)}
        >
          <Calendar size={14} strokeWidth={2} className="df-trigger-icon" />
          <span className="df-trigger-text">{currentLabel}</span>
          <ChevronDown
            size={13}
            strokeWidth={2.5}
            className={`df-trigger-chevron ${open ? "df-trigger-chevron--open" : ""}`}
          />
        </button>

        {open && (
          <div className="df-panel">
            <div className="df-panel-inner">
              {grouped.map((group, gi) => (
                <div key={gi} className="df-group">
                  {group.section && (
                    <div className="df-group-label">{group.section}</div>
                  )}
                  {group.items.map((f) => {
                    const active = value === f.value;
                    return (
                      <button
                        key={f.value}
                        className={`df-item ${active ? "df-item--active" : ""}`}
                        onClick={() => {
                          onChange(f.value);
                          if (f.value !== "custom") setOpen(false);
                        }}
                      >
                        <span className="df-item-label">{f.label}</span>
                        <span className="df-item-check">
                          {active && <Check size={14} strokeWidth={3} />}
                        </span>
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {value === "custom" && (
        <div className="df-custom">
          <input
            type="date"
            className="df-custom-input"
            value={customStart}
            onChange={(e) => onCustomStartChange(e.target.value)}
          />
          <ArrowRight size={12} strokeWidth={2} className="df-custom-arrow" />
          <input
            type="date"
            className="df-custom-input"
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
