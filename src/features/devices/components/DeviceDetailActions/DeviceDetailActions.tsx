import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";
import { Button, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  device: Device;
  onDeviceChange: (device: Device) => void;
  onBack: () => void;
};

export function DeviceDetailActions({ device, onDeviceChange, onBack }: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(device.archivedAt);

  return (
    <>
      <LinkButton to={`/devices/${device.id}/edit`} variant="secondary">
        {t("devices.actions.edit")}
      </LinkButton>
      <Button
        type="button"
        variant="secondary"
        onClick={() => {
          void (async () => {
            const next = isArchived
              ? await devicesApi.unarchive(device.id)
              : await devicesApi.archive(device.id);
            onDeviceChange(next);
          })();
        }}
      >
        {isArchived
          ? t("devices.actions.unarchive")
          : t("devices.actions.archive")}
      </Button>
      <Button type="button" variant="secondary" onClick={onBack}>
        {t("devices.backToList")}
      </Button>
    </>
  );
}
