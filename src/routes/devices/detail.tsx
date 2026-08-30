import { useParams } from "react-router";
import { DeviceDetailPage } from "@/features/devices/pages/DeviceDetailPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export default function DevicesDetailRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return <p className="p-6 text-sm text-red-700">{t("devices.invalidId")}</p>;
  }
  return <DeviceDetailPage deviceId={id} />;
}
