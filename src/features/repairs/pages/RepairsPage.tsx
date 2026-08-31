import { RepairListFilters } from "@/features/repairs/components/RepairListFilters";
import { RepairTable } from "@/features/repairs/components/RepairTable";
import { useRepairList } from "@/features/repairs/hooks/useRepairList";
import type { RepairStatus } from "@/features/repairs/types/repair";
import { REPAIR_STATUSES } from "@/features/repairs/types/repair";
import { Page, PageHeader, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSearchParams } from "react-router";

function statusFromParams(value: string | null): RepairStatus | "" {
  if (!value) {
    return "";
  }
  return (REPAIR_STATUSES as readonly string[]).includes(value)
    ? (value as RepairStatus)
    : "";
}

export function RepairsPage() {
  const { t } = useI18n();
  const [params, setParams] = useSearchParams();
  const initialStatus = statusFromParams(params.get("status"));
  const list = useRepairList({ initialStatus });
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Page>
      <PageHeader
        title={t("repairs.title")}
        description={t("repairs.subtitle")}
      />

      <RepairListFilters
        query={list.query}
        onQueryChange={list.setQuery}
        status={list.status}
        onStatusChange={(value) => {
          list.setStatus(value);
          const next = new URLSearchParams(params);
          if (value) {
            next.set("status", value);
          } else {
            next.delete("status");
          }
          setParams(next, { replace: true });
        }}
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
    </Page>
  );
}
