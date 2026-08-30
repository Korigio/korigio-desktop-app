import { useMemo } from "react";
import { defineChart } from "@tanstack/charts";
import { pie, polar, radialArc } from "@tanstack/charts/polar";
import { Chart } from "@tanstack/charts/react";
import { tooltip } from "@tanstack/charts/tooltip";
import {
  HOME_PIPELINE_STATUSES,
  HOME_PIPELINE_STATUS_COLOR_VAR,
} from "@/features/home/constants";
import type { StatusCount } from "@/features/home/types/dashboard";
import { readCssVar } from "@/features/home/utils/readCssVar";
import { statusCountMap } from "@/features/home/utils/statusCounts";
import { DashboardChartGuard } from "@/features/home/components/DashboardChartGuard";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useTheme } from "@/shared/hooks/useTheme";

type Props = {
  counts: StatusCount[];
};

export function DashboardStatusChart({ counts }: Props) {
  const { t } = useI18n();
  const { resolvedTheme } = useTheme();

  const definition = useMemo(() => {
    const byStatus = statusCountMap(counts);
    const rows = HOME_PIPELINE_STATUSES.map((status) => ({
      status,
      count: byStatus.get(status) ?? 0,
    })).filter((row) => row.count > 0);

    if (rows.length === 0) {
      return null;
    }

    const statuses = rows.map((row) => row.status);
    const statusColors = statuses.map(
      (status) =>
        readCssVar(HOME_PIPELINE_STATUS_COLOR_VAR[status]) ||
        (resolvedTheme === "dark" ? "#9aa8b5" : "#5b6b7c"),
    );
    const surface =
      readCssVar("--color-surface") ||
      (resolvedTheme === "dark" ? "#1c2128" : "#ffffff");
    const arcs = pie(rows, { value: "count" });

    try {
      return defineChart({
        marks: [
          polar({
            radiusRatio: 0.78,
            marks: [
              radialArc(arcs, {
                innerRadius: 56,
                color: "status",
                key: "status",
                stroke: surface,
                strokeWidth: 1,
              }),
            ],
            scales: {
              angle: null,
              radius: null,
            },
          }),
        ],
        scales: {
          x: null,
          y: null,
        },
        color: { domain: statuses, range: statusColors },
        tooltip,
        svgAnimation: false,
      });
    } catch {
      return null;
    }
  }, [counts, resolvedTheme]);

  const empty = (
    <p className="text-sm text-muted">
      {t("home.dashboard.charts.statusEmpty")}
    </p>
  );

  return (
    <Card title={t("home.dashboard.charts.statusTitle")}>
      {definition ? (
        <DashboardChartGuard fallback={empty}>
          <div className="h-60 w-full">
            <Chart
              definition={definition}
              height={240}
              ariaLabel={t("home.dashboard.charts.ariaStatus")}
            />
          </div>
        </DashboardChartGuard>
      ) : (
        empty
      )}
    </Card>
  );
}
