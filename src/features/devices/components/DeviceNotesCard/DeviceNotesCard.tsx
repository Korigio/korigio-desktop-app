import type { Device } from "@/features/devices/types/device";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  device: Device;
};

export function DeviceNotesCard({ device }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("devices.detail.sections.notes")}>
      <DefinitionList
        items={[
          { label: t("devices.fields.notes"), value: device.notes ?? EMPTY },
        ]}
      />
    </Card>
  );
}
