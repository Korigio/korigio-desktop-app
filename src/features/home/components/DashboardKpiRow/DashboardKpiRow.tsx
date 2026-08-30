import { DashboardKpiCard } from "@/features/home/components/DashboardKpiCard";
import type { HomeDashboard } from "@/features/home/types/dashboard";
import {
  countForStatus,
  openPipelineTotal,
} from "@/features/home/utils/statusCounts";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  dashboard: HomeDashboard;
};

export function DashboardKpiRow({ dashboard }: Props) {
  const { t } = useI18n();

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-5">
      <DashboardKpiCard
        label={t("home.dashboard.kpi.receivedToday")}
        value={dashboard.today?.received ?? 0}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.collectedToday")}
        value={dashboard.today?.collected ?? 0}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.ready")}
        value={countForStatus(dashboard.statusCounts ?? [], "ready")}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.open")}
        value={openPipelineTotal(dashboard.statusCounts ?? [])}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.stale")}
        value={dashboard.staleCount ?? 0}
      />
    </div>
  );
}
