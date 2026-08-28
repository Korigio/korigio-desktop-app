import type { Device } from "@/features/devices/types/device";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  device: Device;
};

export function DeviceInfoCard({ device }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("devices.detail.sections.info")}>
      <DefinitionList
        items={[
          {
            label: t("devices.fields.deviceType"),
            value: device.deviceType ?? EMPTY,
          },
          {
            label: t("devices.fields.manufacturer"),
            value: device.manufacturer ?? EMPTY,
          },
          { label: t("devices.fields.model"), value: device.model ?? EMPTY },
          {
            label: t("devices.fields.serialNumber"),
            value: device.serialNumber ?? EMPTY,
          },
          {
            label: t("devices.fields.accessories"),
            value: device.accessories ?? EMPTY,
          },
        ]}
      />
    </Card>
  );
}
