import { LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function NewDeviceButton({ customerId }: { customerId?: number }) {
  const { t } = useI18n();
  const to = customerId
    ? `/devices/new?customerId=${customerId}`
    : "/devices/new";
  return <LinkButton to={to}>{t("devices.actions.new")}</LinkButton>;
}
