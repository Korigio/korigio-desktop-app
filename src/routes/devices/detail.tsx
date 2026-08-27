import { useParams } from "react-router";
import { DeviceDetailPage } from "@/features/devices/pages/DeviceDetailPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function DevicesDetailRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return <p className="p-6 text-sm text-red-700">{t("devices.invalidId")}</p>;
  }
  return <DeviceDetailPage deviceId={id} />;
}
