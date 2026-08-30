import { useParams } from "react-router";
import { DeviceEditPage } from "@/features/devices/pages/DeviceEditPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export default function DevicesEditRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return <p className="p-6 text-sm text-red-700">{t("devices.invalidId")}</p>;
  }
  return <DeviceEditPage deviceId={id} />;
}
