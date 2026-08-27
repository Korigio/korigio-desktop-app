import { RepairListFilters } from "@/features/repairs/components/RepairListFilters";
import { RepairTable } from "@/features/repairs/components/RepairTable";
import { NewRepairButton } from "@/features/repairs/components/NewRepairButton";
import { useRepairList } from "@/features/repairs/hooks/useRepairList";
import { Page, PageHeader, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function RepairsPage() {
  const { t } = useI18n();
  const list = useRepairList();
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Page>
      <PageHeader
        title={t("repairs.title")}
        description={t("repairs.subtitle")}
        actions={<NewRepairButton />}
      />

      <RepairListFilters
        query={list.query}
        onQueryChange={list.setQuery}
        status={list.status}
        onStatusChange={list.setStatus}
      />

      {list.error ? (
        <StatusMessage tone="danger">{list.error}</StatusMessage>
      ) : null}

      {list.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : (
        <RepairTable
          repairs={list.result?.items ?? []}
          statusUpdatingId={list.statusUpdatingId}
          onStatusChange={(repair, nextStatus) => {
            void list.updateStatus(repair, nextStatus);
          }}
        />
      )}

      <PaginationBar
        page={list.page}
        totalPages={totalPages}
        onPrevious={() => list.setPage(list.page - 1)}
        onNext={() => list.setPage(list.page + 1)}
      />
    </Page>
  );
}
