import { LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function NewRepairButton({
  customerId,
  deviceId,
}: {
  customerId?: string;
  deviceId?: string;
}) {
  const { t } = useI18n();
  const params = new URLSearchParams();
  if (customerId) {
    params.set("customerId", String(customerId));
  }
  if (deviceId) {
    params.set("deviceId", String(deviceId));
  }
  const to = params.toString() ? `/repairs/new?${params}` : "/repairs/new";
  return <LinkButton to={to}>{t("repairs.actions.new")}</LinkButton>;
}
