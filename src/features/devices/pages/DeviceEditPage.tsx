import { DeviceForm } from "@/features/devices/components/DeviceForm";
import { DeviceLoadState } from "@/features/devices/components/DeviceLoadState";
import { useDeviceDetail } from "@/features/devices/hooks/useDeviceDetail";
import { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import type { Device } from "@/features/devices/types/device";
import { Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { deviceId: string };

export function DeviceEditPage({ deviceId }: Props) {
  const { t } = useI18n();
  const { device, loading, error } = useDeviceDetail(deviceId);

  if (loading) {
    return <DeviceLoadState />;
  }

  if (error || !device) {
    return <DeviceLoadState message={error ?? t("devices.notFound")} />;
  }

  return <DeviceEditForm device={device} />;
}

function DeviceEditForm({ device }: { device: Device }) {
  const { t } = useI18n();
  const form = useDeviceForm({ mode: "edit", device });
  const isArchived = Boolean(device.archivedAt);

  return (
    <Page>
      <PageHeader title={t("devices.editTitle")} />
      <DeviceForm
        form={form}
        submitLabel={t("devices.actions.save")}
        disabled={isArchived}
        lockCustomer
      />
      {isArchived ? (
        <StatusMessage>{t("devices.archivedEditHint")}</StatusMessage>
      ) : null}
    </Page>
  );
}
