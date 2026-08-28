import { RepairListFilters } from "@/features/repairs/components/RepairListFilters";
import { RepairTable } from "@/features/repairs/components/RepairTable";
import { NewRepairButton } from "@/features/repairs/components/NewRepairButton";
import { useRepairList } from "@/features/repairs/hooks/useRepairList";
import { Card, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  customerId: number;
  deviceId: number;
};

/** Repairs for one device — used on device detail. */
export function DeviceRepairsSection({ customerId, deviceId }: Props) {
  const { t } = useI18n();
  const list = useRepairList({ customerId, deviceId });
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Card
      title={t("repairs.deviceSectionTitle")}
      actions={
        <NewRepairButton customerId={customerId} deviceId={deviceId} />
      }
    >
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
          <RepairTable
            repairs={list.result?.items ?? []}
            showCustomerLink={false}
            showDeviceLink={false}
          />
        )}

        <PaginationBar
          page={list.page}
          totalPages={totalPages}
          onPrevious={() => list.setPage(list.page - 1)}
          onNext={() => list.setPage(list.page + 1)}
        />
      </div>
    </Card>
  );
}
