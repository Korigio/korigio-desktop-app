import { DashboardRepairList } from "@/features/home/components/DashboardRepairList";
import { DashboardStatusCounts } from "@/features/home/components/DashboardStatusCounts";
import { useHomeDashboard } from "@/features/home/hooks/useHomeDashboard";
import { Page, PageHeader, StatusMessage } from "@/ui";
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
        <div className="flex flex-col gap-6 sm:gap-8">
          <p className="text-sm text-muted">
            {t("home.dashboard.todayLine")
              .replace("{received}", String(dashboard.today.received))
              .replace("{collected}", String(dashboard.today.collected))}
          </p>

          <DashboardStatusCounts counts={dashboard.statusCounts} />

          <div className="grid grid-cols-1 gap-6 md:grid-cols-2 md:gap-8">
            <DashboardRepairList
              title={t("home.dashboard.readyTitle")}
              empty={t("home.dashboard.readyEmpty")}
              rows={dashboard.readyForPickup}
              showReadyAt
            />
            <DashboardRepairList
              title={t("home.dashboard.staleTitle").replace(
                "{days}",
                String(dashboard.staleAfterDays),
              )}
              empty={t("home.dashboard.staleEmpty")}
              rows={dashboard.staleRepairs}
            />
          </div>
        </div>
      ) : null}
    </Page>
  );
}
