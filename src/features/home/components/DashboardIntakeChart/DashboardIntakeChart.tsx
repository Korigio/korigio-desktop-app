import { useMemo } from "react";
import { barY, defineChart, group } from "@tanstack/charts";
import { Chart } from "@tanstack/charts/react";
import { scaleBand } from "@tanstack/charts/scales/band";
import { scaleLinear } from "@tanstack/charts/scales/linear";
import { tooltip } from "@tanstack/charts/tooltip";
import type { IntakeDayCounts } from "@/features/home/types/dashboard";
import {
  intakeChartRows,
  isIntakeChartEmpty,
} from "@/features/home/utils/intakeChartRows";
import { readCssVar } from "@/features/home/utils/readCssVar";
import { weekdayLabel } from "@/features/home/utils/weekdayLabel";
import { DashboardChartGuard } from "@/features/home/components/DashboardChartGuard";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useTheme } from "@/shared/hooks/useTheme";

type Props = {
  intakeByDay?: IntakeDayCounts[] | null;
};

const EMPTY_DAYS: IntakeDayCounts[] = [];

export function DashboardIntakeChart({ intakeByDay }: Props) {
  const { t, locale } = useI18n();
  const { resolvedTheme } = useTheme();
  const days = intakeByDay ?? EMPTY_DAYS;
  const isEmpty = isIntakeChartEmpty(days);

  const definition = useMemo(() => {
    if (isEmpty) {
      return null;
    }

    const receivedLabel = t("home.dashboard.charts.received");
    const collectedLabel = t("home.dashboard.charts.collected");
    const rows = intakeChartRows(days, (date) =>
      weekdayLabel(date, locale),
    ).map((row) => ({
      ...row,
      series: row.series === "received" ? receivedLabel : collectedLabel,
    }));
    const labels = [...new Set(rows.map((row) => row.label))];
    const receivedColor =
      readCssVar("--color-primary") ||
      (resolvedTheme === "dark" ? "#2ee6c5" : "#0f9d8a");
    const collectedColor =
      readCssVar("--color-status-ready") ||
      (resolvedTheme === "dark" ? "#4ee090" : "#2db36b");

    try {
      return defineChart({
        marks: [
          barY(rows, {
            x: "label",
            y: "value",
            z: "series",
            color: "series",
            layout: group({ padding: 0.15 }),
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
        color: {
          domain: [receivedLabel, collectedLabel],
          range: [receivedColor, collectedColor],
        },
        tooltip,
        svgAnimation: false,
      });
    } catch {
      return null;
    }
  }, [days, isEmpty, locale, resolvedTheme, t]);

  const empty = (
    <p className="text-sm text-muted">
      {t("home.dashboard.charts.intakeEmpty")}
    </p>
  );

  return (
    <Card title={t("home.dashboard.charts.intakeTitle")}>
      {!isEmpty && definition ? (
        <DashboardChartGuard fallback={empty}>
          <div className="h-60 w-full">
            <Chart
              definition={definition}
              height={240}
              ariaLabel={t("home.dashboard.charts.ariaIntake")}
            />
          </div>
        </DashboardChartGuard>
      ) : (
        empty
      )}
    </Card>
  );
}
