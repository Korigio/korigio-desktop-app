import { DashboardIntakeChart } from "@/features/home/components/DashboardIntakeChart";
import { DashboardKpiRow } from "@/features/home/components/DashboardKpiRow";
import { DashboardRepairList } from "@/features/home/components/DashboardRepairList";
import { DashboardRevenueChart } from "@/features/home/components/DashboardRevenueChart";
import { DashboardRevenueKpiRow } from "@/features/home/components/DashboardRevenueKpiRow";
import { DashboardStatusChart } from "@/features/home/components/DashboardStatusChart";
import { DashboardStatusCounts } from "@/features/home/components/DashboardStatusCounts";
import { useHomeDashboard } from "@/features/home/hooks/useHomeDashboard";
import { Card, Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function HomePage() {
  const { t } = useI18n();
  const { dashboard, loading, error } = useHomeDashboard();

  return (
    <Page className="max-w-6xl">
      <PageHeader
        title={t("home.dashboard.title")}
        description={t("home.dashboard.subtitle")}
      />

      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
      {loading && !dashboard ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : null}

      {dashboard ? (
        <>
          <DashboardKpiRow dashboard={dashboard} />
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
          <DashboardStatusCounts counts={dashboard.statusCounts} />
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <Card title={t("home.dashboard.readyTitle")}>
              <DashboardRepairList
                empty={t("home.dashboard.readyEmpty")}
                rows={dashboard.readyForPickup ?? []}
                showReadyAt
              />
            </Card>
            <Card
              title={t("home.dashboard.staleTitle").replace(
                "{days}",
                String(dashboard.staleAfterDays ?? 7),
              )}
            >
              <DashboardRepairList
                empty={t("home.dashboard.staleEmpty")}
                rows={dashboard.staleRepairs ?? []}
              />
            </Card>
          </div>
        </>
      ) : null}
    </Page>
  );
}
