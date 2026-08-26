import { DeviceDetailActions } from "@/features/devices/components/DeviceDetailActions";
import { DeviceDetailFields } from "@/features/devices/components/DeviceDetailFields";
import { DeviceLoadState } from "@/features/devices/components/DeviceLoadState";
import { useDeviceDetail } from "@/features/devices/hooks/useDeviceDetail";
import { deviceLabel } from "@/features/devices/types/device";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { deviceId: number };

export function DeviceDetailPage({ deviceId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { device, loading, error, setDevice } = useDeviceDetail(deviceId);

  if (loading) {
    return <DeviceLoadState />;
  }

  if (error || !device) {
    return <DeviceLoadState message={error ?? t("devices.notFound")} />;
  }

  return (
    <Page>
      <PageHeader
        title={deviceLabel(device)}
        description={
          device.archivedAt
            ? t("devices.status.archived")
            : t("devices.status.active")
        }
        actions={
          <DeviceDetailActions
            device={device}
            onDeviceChange={setDevice}
            onBack={() => navigate("/devices")}
          />
        }
      />
      <DeviceDetailFields device={device} />
    </Page>
  );
}
