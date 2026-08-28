import { useShopSettings } from "@/features/settings/hooks/useShopSettings";
import { Button, FormField, StatusMessage, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function ShopSettingsFields() {
  const { t } = useI18n();
  const shop = useShopSettings();

  if (shop.loading) {
    return <StatusMessage>{t("common.loading")}</StatusMessage>;
  }

  return (
    <div className="flex max-w-md flex-col gap-3">
      <h2 className="text-sm font-medium">{t("settings.shop.title")}</h2>
      <p className="text-sm text-muted">{t("settings.shop.subtitle")}</p>
      <FormField
        label={t("settings.shop.fields.taxRatePercent")}
        htmlFor="settings-tax-rate"
      >
        <TextField
          id="settings-tax-rate"
          inputMode="decimal"
          value={shop.taxRatePercent}
          disabled={shop.saving}
          onChange={(event) => shop.setTaxRatePercent(event.target.value)}
        />
      </FormField>
      <FormField
        label={t("settings.shop.fields.currency")}
        htmlFor="settings-currency"
      >
        <TextField
          id="settings-currency"
          value={shop.currency}
          disabled={shop.saving}
          maxLength={3}
          onChange={(event) => shop.setCurrency(event.target.value)}
        />
      </FormField>
      <p className="text-sm text-muted">{t("settings.shop.hint")}</p>
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
