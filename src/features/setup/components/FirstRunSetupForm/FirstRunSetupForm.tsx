import { CompanyForm } from "@/features/companies/components/CompanyForm";
import { ShopTaxCurrencyFields } from "@/features/settings/components/ShopTaxCurrencyFields";
import type { useFirstRunForm } from "@/features/setup/hooks/useFirstRunForm";
import { Button, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  setup: ReturnType<typeof useFirstRunForm>;
};

export function FirstRunSetupForm({ setup }: Props) {
  const { t } = useI18n();
  const { form, error, ready } = setup;

  if (!ready) {
    return <StatusMessage>{t("common.loading")}</StatusMessage>;
  }

  return (
    <CompanyForm form={form} hideSubmit>
      <form.Subscribe
        selector={(state) =>
          [
            state.values.taxRatePercent,
            state.values.currency,
            state.canSubmit,
            state.isSubmitting,
          ] as const
        }
      >
        {([taxRatePercent, currency, canSubmit, isSubmitting]) => (
          <>
            <ShopTaxCurrencyFields
              taxRatePercent={taxRatePercent}
              currency={currency}
              onTaxRatePercentChange={(value) =>
                form.setFieldValue("taxRatePercent", value)
              }
              onCurrencyChange={(value) =>
                form.setFieldValue("currency", value)
              }
              disabled={isSubmitting}
              idPrefix="setup"
            />
            {error ? (
              <StatusMessage tone="danger">{error}</StatusMessage>
            ) : null}
            <div>
              <Button type="submit" disabled={!canSubmit || isSubmitting}>
                {isSubmitting ? t("common.saving") : t("setup.actions.submit")}
              </Button>
            </div>
            <p className="text-sm text-muted">{t("setup.backupHint")}</p>
          </>
        )}
      </form.Subscribe>
    </CompanyForm>
  );
}
