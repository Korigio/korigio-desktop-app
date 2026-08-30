import { useMemo } from "react";
import { barY, defineChart } from "@tanstack/charts";
import { Chart } from "@tanstack/charts/react";
import { scaleBand } from "@tanstack/charts/scales/band";
import { scaleLinear } from "@tanstack/charts/scales/linear";
import { tooltip } from "@tanstack/charts/tooltip";
import { DashboardChartGuard } from "@/features/home/components/DashboardChartGuard";
import type { RevenueDayTotals } from "@/features/home/types/dashboard";
import { readCssVar } from "@/features/home/utils/readCssVar";
import {
  isRevenueChartEmpty,
  revenueChartRows,
} from "@/features/home/utils/revenueChartRows";
import { weekdayLabel } from "@/features/home/utils/weekdayLabel";
import { formatMoneyCents } from "@/features/repairs/utils/money";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useTheme } from "@/shared/hooks/useTheme";

type Props = {
  revenueByDay?: RevenueDayTotals[] | null;
  currency?: string | null;
};

const EMPTY_DAYS: RevenueDayTotals[] = [];

export function DashboardRevenueChart({ revenueByDay, currency }: Props) {
  const { t, locale } = useI18n();
  const { resolvedTheme } = useTheme();
  const days = revenueByDay ?? EMPTY_DAYS;
  const resolvedCurrency = currency ?? "EUR";
  const isEmpty = isRevenueChartEmpty(days);

  const definition = useMemo(() => {
    if (isEmpty) {
      return null;
    }

    const rows = revenueChartRows(days, (date) => weekdayLabel(date, locale));
    const labels = rows.map((row) => row.label);
    const barColor =
      readCssVar("--color-primary") ||
      (resolvedTheme === "dark" ? "#2ee6c5" : "#0f9d8a");

    try {
      return defineChart({
        marks: [
          barY(rows, {
            x: "label",
            y: "value",
            fill: barColor,
            radius: 4,
          }),
        ],
        scales: {
          x: {
            scale: () => scaleBand<string>().domain(labels).padding(0.2),
          },
          y: {
            scale: scaleLinear,
            nice: true,
            grid: true,
          },
        },
        tooltip: {
          use: tooltip,
          format(point) {
            return `${point.datum.label}: ${formatMoneyCents(
              point.datum.cents,
              resolvedCurrency,
              locale,
            )}`;
          },
        },
        svgAnimation: false,
      });
    } catch {
      return null;
    }
  }, [days, isEmpty, locale, resolvedCurrency, resolvedTheme]);

  const empty = (
    <p className="text-sm text-muted">
      {t("home.dashboard.charts.revenueEmpty")}
    </p>
  );

  return (
    <Card title={t("home.dashboard.charts.revenueTitle")}>
      {!isEmpty && definition ? (
        <DashboardChartGuard fallback={empty}>
          <div className="h-60 w-full">
            <Chart
              definition={definition}
              height={240}
              ariaLabel={t("home.dashboard.charts.ariaRevenue")}
            />
          </div>
        </DashboardChartGuard>
      ) : (
        empty
      )}
    </Card>
  );
}
