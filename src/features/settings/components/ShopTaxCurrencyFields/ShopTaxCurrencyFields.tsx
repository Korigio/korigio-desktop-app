import {
  currencyOptionLabel,
  currencySelectOptions,
} from "@/features/settings/constants/currencies";
import { FormField, SelectField, TextField } from "@/ui";
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
  const currencyOptions = currencySelectOptions(currency);
  const currencyValue = currency.trim().toUpperCase();

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
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
        <SelectField
          id={currencyId}
          value={currencyValue}
          disabled={disabled}
          onChange={(event) => onCurrencyChange(event.target.value)}
        >
          <option value="">
            {t("settings.shop.fields.currencyPlaceholder")}
          </option>
          {currencyOptions.map((option) => (
            <option key={option.code} value={option.code}>
              {currencyOptionLabel(option)}
            </option>
          ))}
        </SelectField>
      </FormField>
      <p className="text-sm text-muted sm:col-span-2">
        {t("settings.shop.hint")}
      </p>
    </div>
  );
}
