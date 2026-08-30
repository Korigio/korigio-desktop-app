import { DashboardKpiCard } from "@/features/home/components/DashboardKpiCard";
import type { HomeDashboard } from "@/features/home/types/dashboard";
import { formatMoneyCents } from "@/features/repairs/utils/money";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  dashboard: HomeDashboard;
};

export function DashboardRevenueKpiRow({ dashboard }: Props) {
  const { t, locale } = useI18n();
  const currency = dashboard.currency ?? "EUR";
  const revenue = dashboard.revenue;
  if (!revenue) {
    return null;
  }

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
      <DashboardKpiCard
        label={t("home.dashboard.kpi.collectedRevenueToday")}
        value={formatMoneyCents(
          revenue.collectedGrossCentsToday,
          currency,
          locale,
        )}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.collectedRevenueWeek")}
        value={formatMoneyCents(
          revenue.collectedGrossCentsWeek,
          currency,
          locale,
        )}
      />
      <DashboardKpiCard
        label={t("home.dashboard.kpi.openEstimate")}
        value={formatMoneyCents(revenue.openEstimateGrossCents, currency, locale)}
      />
    </div>
  );
}
