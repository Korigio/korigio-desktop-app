import type { IntakeDayCounts } from "@/features/home/types/dashboard";

export type IntakeChartSeries = "received" | "collected";

export type IntakeChartRow = {
  label: string;
  series: IntakeChartSeries;
  value: number;
};

export function intakeChartRows(
  days: IntakeDayCounts[],
  formatLabel: (date: string) => string,
): IntakeChartRow[] {
  return days.flatMap((day) => [
    {
      label: formatLabel(day.date),
      series: "received",
      value: day.received,
    },
    {
      label: formatLabel(day.date),
      series: "collected",
      value: day.collected,
    },
  ]);
}

export function isIntakeChartEmpty(
  days: IntakeDayCounts[] | null | undefined,
): boolean {
  return (
    !days ||
    days.length === 0 ||
    days.every((day) => day.received === 0 && day.collected === 0)
  );
}
