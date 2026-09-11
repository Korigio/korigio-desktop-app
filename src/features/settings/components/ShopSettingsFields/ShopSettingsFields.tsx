import { ShopTaxCurrencyFields } from "@/features/settings/components/ShopTaxCurrencyFields";
import { useShopSettings } from "@/features/settings/hooks/useShopSettings";
import { Button, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function ShopSettingsFields() {
  const { t } = useI18n();
  const shop = useShopSettings();

  if (shop.loading) {
    return <StatusMessage>{t("common.loading")}</StatusMessage>;
  }

  return (
    <div className="flex max-w-xl flex-col gap-3">
      <ShopTaxCurrencyFields
        taxRatePercent={shop.taxRatePercent}
        currency={shop.currency}
        onTaxRatePercentChange={shop.setTaxRatePercent}
        onCurrencyChange={shop.setCurrency}
        disabled={shop.saving}
        idPrefix="settings"
      />
      <div>
        <Button
          type="button"
          disabled={shop.saving}
          onClick={() => void shop.save()}
        >
          {shop.saving ? t("common.saving") : t("settings.shop.actions.save")}
        </Button>
      </div>
      {shop.error ? (
        <StatusMessage tone="danger">{shop.error}</StatusMessage>
      ) : null}
      {shop.success ? (
        <StatusMessage tone="success">{shop.success}</StatusMessage>
      ) : null}
    </div>
  );
}
