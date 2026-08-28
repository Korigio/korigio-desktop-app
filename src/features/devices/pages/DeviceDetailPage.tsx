import { DeviceCustomerCard } from "@/features/devices/components/DeviceCustomerCard";
import { DeviceDetailActions } from "@/features/devices/components/DeviceDetailActions";
import { DeviceDetailStatusPanel } from "@/features/devices/components/DeviceDetailStatusPanel";
import { DeviceInfoCard } from "@/features/devices/components/DeviceInfoCard";
import { DeviceLoadState } from "@/features/devices/components/DeviceLoadState";
import { DeviceNotesCard } from "@/features/devices/components/DeviceNotesCard";
import { useDeviceDetail } from "@/features/devices/hooks/useDeviceDetail";
import { useDeviceDetailRelations } from "@/features/devices/hooks/useDeviceDetailRelations";
import { deviceLabel } from "@/features/devices/types/device";
import { DeviceRepairsSection } from "@/features/repairs/components/DeviceRepairsSection";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { deviceId: number };

export function DeviceDetailPage({ deviceId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { device, loading, error, setDevice } = useDeviceDetail(deviceId);
  const relations = useDeviceDetailRelations(device);

  if (loading) {
    return <DeviceLoadState />;
  }

  if (error || !device) {
    return <DeviceLoadState message={error ?? t("devices.notFound")} />;
  }

  return (
    <Page className="max-w-5xl">
      <PageHeader
        title={deviceLabel(device)}
        description={t("devices.detail.subtitle")}
        actions={
          <DeviceDetailActions
            device={device}
            onDeviceChange={setDevice}
            onBack={() => navigate("/devices")}
          />
        }
      />

      <div className="flex flex-col gap-4">
        <DeviceDetailStatusPanel device={device} />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 lg:gap-6">
          <DeviceInfoCard device={device} />
          <DeviceCustomerCard
            device={device}
            customer={relations.customer}
            loading={relations.loading}
          />
          <DeviceNotesCard device={device} />
        </div>

        <DeviceRepairsSection
          customerId={device.customerId}
          deviceId={device.id}
        />
      </div>
    </Page>
  );
}
