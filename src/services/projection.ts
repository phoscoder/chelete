export type ProjectionPeriodType = "weeks" | "months" | "years";

export interface ProjectionInput {
  startingBalance: number;
  income: number;
  expense: number;
  periodType: ProjectionPeriodType;
  periods: number;
}

export interface ProjectionPoint {
  step: number;
  label: string;
  balance: number;
}

export interface ProjectionSummary {
  netPerPeriod: number;
  finalBalance: number;
  totalIncrease: number;
  retentionRate: number | null;
  projectedDate: string | null;
}

export interface ProjectionResult {
  points: ProjectionPoint[];
  summary: ProjectionSummary;
  maxPeriods: number;
}

export const PERIOD_UNIT_LABELS: Record<ProjectionPeriodType, string> = {
  weeks: "week",
  months: "month",
  years: "year",
};

export const PERIOD_PLURAL_LABELS: Record<ProjectionPeriodType, string> = {
  weeks: "weeks",
  months: "months",
  years: "years",
};

export const PERIOD_LIMITS: Record<ProjectionPeriodType, number> = {
  weeks: 260,
  months: 600,
  years: 60,
};

const PERIOD_DAYS: Record<ProjectionPeriodType, number> = {
  weeks: 7,
  months: 30,
  years: 365,
};

export function periodLabel(
  periodType: ProjectionPeriodType,
  step: number
): string {
  if (step === 0) return "Now";
  const unit = PERIOD_UNIT_LABELS[periodType];
  return step === 1 ? `1 ${unit}` : `${step} ${unit}s`;
}

export type SubscriptionFrequency =
  | "weekly"
  | "bi_weekly"
  | "monthly"
  | "quarterly"
  | "bi_yearly"
  | "yearly";

const SUBSCRIPTION_PERIODS_PER_YEAR: Record<SubscriptionFrequency, number> = {
  weekly: 52,
  bi_weekly: 26,
  monthly: 12,
  quarterly: 4,
  bi_yearly: 2,
  yearly: 1,
};

const PROJECTION_PERIODS_PER_YEAR: Record<ProjectionPeriodType, number> = {
  weeks: 52,
  months: 12,
  years: 1,
};

export function normalizeSubscriptionToPeriod(
  amount: number,
  frequency: string,
  periodType: ProjectionPeriodType
): number {
  const perYear =
    SUBSCRIPTION_PERIODS_PER_YEAR[frequency as SubscriptionFrequency];
  if (!perYear) return 0;
  return Math.round((amount * perYear) / PROJECTION_PERIODS_PER_YEAR[periodType]);
}

export function computeProjection(input: ProjectionInput): ProjectionResult {
  const { startingBalance, income, expense, periodType, periods } = input;
  const netPerPeriod = income - expense;
  const maxPeriods = PERIOD_LIMITS[periodType];

  const points: ProjectionPoint[] = [];
  const balance0 = startingBalance;
  points.push({
    step: 0,
    label: periodLabel(periodType, 0),
    balance: balance0,
  });

  for (let step = 1; step <= periods; step++) {
    points.push({
      step,
      label: periodLabel(periodType, step),
      balance: startingBalance + netPerPeriod * step,
    });
  }

  const finalBalance = startingBalance + netPerPeriod * periods;
  const totalIncrease = finalBalance - startingBalance;

  const today = new Date();
  today.setHours(0, 0, 0, 0);
  today.setDate(today.getDate() + PERIOD_DAYS[periodType] * periods);
  const projectedDate = today.toISOString().split("T")[0];

  return {
    points,
    summary: {
      netPerPeriod,
      finalBalance,
      totalIncrease,
      retentionRate: income > 0 ? (netPerPeriod / income) * 100 : null,
      projectedDate,
    },
    maxPeriods,
  };
}