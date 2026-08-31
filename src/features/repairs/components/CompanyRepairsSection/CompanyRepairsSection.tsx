import { RepairListFilters } from "@/features/repairs/components/RepairListFilters";
import { RepairTable } from "@/features/repairs/components/RepairTable";
import { useRepairList } from "@/features/repairs/hooks/useRepairList";
import { Card, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  companyId: string;
};

/** Repairs belonging to one company — used on company detail. */
export function CompanyRepairsSection({ companyId }: Props) {
  const { t } = useI18n();
  const list = useRepairList({ companyId });
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Card title={t("repairs.companySectionTitle")}>
      <div className="flex flex-col gap-4">
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
          <RepairTable repairs={list.result?.items ?? []} />
        )}

        <PaginationBar
          page={list.page}
          totalPages={totalPages}
          onPrevious={() => list.setPage(list.page - 1)}
          onNext={() => list.setPage(list.page + 1)}
          pageSize={list.pageSize}
          onPageSizeChange={list.setPageSize}
        />
      </div>
    </Card>
  );
}
