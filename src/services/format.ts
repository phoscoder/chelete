export function formatMoney(amount: number, _currency = "USD"): string {
  const abs = Math.abs(amount);
  const dollars = abs / 100;
  const formatted = dollars.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  const sign = amount >= 0 ? "+" : "-";
  return `${sign}$${formatted}`;
}

export function formatMoneyShort(amount: number): string {
  const abs = Math.abs(amount);
  const dollars = abs / 100;
  if (dollars >= 1000) {
    return `$${(dollars / 1000).toFixed(1)}k`;
  }
  return `$${dollars.toFixed(2)}`;
}

export function formatBalance(amount: number): string {
  const dollars = amount / 100;
  return `$${dollars.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

export function formatDate(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}

export function formatDateFull(dateStr: string): string {
  const d = new Date(dateStr);
  return d.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function todayISO(): string {
  return new Date().toISOString().split("T")[0];
}

export function currentMonth(): string {
  const d = new Date();
  return d.toLocaleDateString("en-US", { year: "numeric", month: "long" });
}

export function accountTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    cash: "Cash",
    bank: "Bank",
    savings: "Savings",
    credit_card: "Credit Card",
    mobile_money: "Mobile Money",
    investment: "Investment",
    other: "Other",
  };
  return labels[type] || type;
}
