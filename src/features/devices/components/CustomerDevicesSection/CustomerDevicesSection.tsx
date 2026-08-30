import { DeviceListFilters } from "@/features/devices/components/DeviceListFilters";
import { DeviceTable } from "@/features/devices/components/DeviceTable";
import { NewDeviceButton } from "@/features/devices/components/NewDeviceButton";
import { useDeviceList } from "@/features/devices/hooks/useDeviceList";
import { Card, PaginationBar, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  customerId: string;
};

/** Devices belonging to one customer — used on customer detail. */
export function CustomerDevicesSection({ customerId }: Props) {
  const { t } = useI18n();
  const list = useDeviceList({ customerId });
  const totalPages = list.result
    ? Math.max(1, Math.ceil(list.result.total / list.result.pageSize))
    : 1;

  return (
    <Card
      title={t("devices.customerSectionTitle")}
      actions={<NewDeviceButton customerId={customerId} />}
    >
      <div className="flex flex-col gap-4">
        <DeviceListFilters
          query={list.query}
          onQueryChange={list.setQuery}
          includeArchived={list.includeArchived}
          onIncludeArchivedChange={list.setIncludeArchived}
        />

        {list.error ? (
          <StatusMessage tone="danger">{list.error}</StatusMessage>
        ) : null}

        {list.loading ? (
          <StatusMessage>{t("common.loading")}</StatusMessage>
        ) : (
          <DeviceTable
            devices={list.result?.items ?? []}
            showCustomerLink={false}
            onArchive={(device) => void list.archive(device)}
            onUnarchive={(device) => void list.unarchive(device)}
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
