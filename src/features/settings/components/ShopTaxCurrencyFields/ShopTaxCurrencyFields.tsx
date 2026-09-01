import { FormField, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  taxRatePercent: string;
  currency: string;
  onTaxRatePercentChange: (value: string) => void;
  onCurrencyChange: (value: string) => void;
  disabled?: boolean;
  idPrefix: string;
};

export function ShopTaxCurrencyFields({
  taxRatePercent,
  currency,
  onTaxRatePercentChange,
  onCurrencyChange,
  disabled,
  idPrefix,
}: Props) {
  const { t } = useI18n();
  const taxId = `${idPrefix}-tax-rate`;
  const currencyId = `${idPrefix}-currency`;

  return (
    <>
      <FormField
        label={t("settings.shop.fields.taxRatePercent")}
        htmlFor={taxId}
      >
        <TextField
          id={taxId}
          inputMode="decimal"
          value={taxRatePercent}
          disabled={disabled}
          onChange={(event) => onTaxRatePercentChange(event.target.value)}
        />
      </FormField>
      <FormField
        label={t("settings.shop.fields.currency")}
        htmlFor={currencyId}
      >
        <TextField
          id={currencyId}
          value={currency}
          disabled={disabled}
          maxLength={3}
          onChange={(event) => onCurrencyChange(event.target.value)}
        />
      </FormField>
      <p className="text-sm text-muted">{t("settings.shop.hint")}</p>
    </>
  );
}
