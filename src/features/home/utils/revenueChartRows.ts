import type { RevenueDayTotals } from "@/features/home/types/dashboard";

export type RevenueChartRow = {
  label: string;
  value: number;
  cents: number;
};

export function revenueChartRows(
  days: RevenueDayTotals[],
  formatLabel: (date: string) => string,
): RevenueChartRow[] {
  return days.map((day) => ({
    label: formatLabel(day.date),
    value: day.collectedGrossCents / 100,
    cents: day.collectedGrossCents,
  }));
}

export function isRevenueChartEmpty(
  days: RevenueDayTotals[] | null | undefined,
): boolean {
  return (
    !days ||
    days.length === 0 ||
    days.every((day) => day.collectedGrossCents === 0)
  );
}
