import { DashboardIntakeChart } from "@/features/home/components/DashboardIntakeChart";
import { DashboardRevenueChart } from "@/features/home/components/DashboardRevenueChart";
import { DashboardRevenueKpiRow } from "@/features/home/components/DashboardRevenueKpiRow";
import { DashboardStatusChart } from "@/features/home/components/DashboardStatusChart";
import { DashboardStatusCounts } from "@/features/home/components/DashboardStatusCounts";
import { useHomeDashboard } from "@/features/home/hooks/useHomeDashboard";
import { Page, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function HomePage() {
  const { t } = useI18n();
  const { dashboard, loading, error } = useHomeDashboard();

  return (
    <Page className="max-w-6xl">
      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
      {loading && !dashboard ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : null}

      {dashboard ? (
        <>
          <DashboardStatusCounts counts={dashboard.statusCounts} />
          {dashboard.revenue != null ? (
            <DashboardRevenueKpiRow dashboard={dashboard} />
          ) : null}
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <DashboardStatusChart counts={dashboard.statusCounts} />
            <DashboardIntakeChart intakeByDay={dashboard.intakeByDay ?? []} />
          </div>
          {dashboard.revenue != null ? (
            <DashboardRevenueChart
              currency={dashboard.currency}
              revenueByDay={dashboard.revenueByDay}
            />
          ) : null}
        </>
      ) : null}
    </Page>
  );
}
