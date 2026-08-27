import type { Device } from "@/features/devices/types/device";
import { DefinitionList, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  device: Device;
};

export function DeviceDetailFields({ device }: Props) {
  const { t } = useI18n();

  return (
    <DefinitionList
      items={[
        {
          label: t("devices.fields.customer"),
          value: (
            <LinkButton
              to={`/customers/${device.customerId}`}
              variant="secondary"
              className="px-2 py-1 text-xs"
            >
              #{device.customerId}
            </LinkButton>
          ),
        },
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
        { label: t("devices.fields.notes"), value: device.notes ?? EMPTY },
      ]}
    />
  );
}
