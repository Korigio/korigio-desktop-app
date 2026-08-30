import { ShopSettingsFields } from "@/features/settings/components/ShopSettingsFields";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function ShopSettingsPage() {
  const { t } = useI18n();

  return (
    <Card title={t("settings.shop.title")}>
      <ShopSettingsFields />
    </Card>
  );
}
