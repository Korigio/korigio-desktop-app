import { useEffect, useState } from "react";
import type { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import { RepairEstimateFields } from "@/features/repairs/components/RepairEstimateFields";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type { ShopSettings } from "@/features/settings/types/shopSettings";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  form: ReturnType<typeof useRepairForm>;
  disabled?: boolean;
  shopSettings?: ShopSettings | null;
};

export function IntakeEstimateFields({
  form,
  disabled = false,
  shopSettings: shopSettingsProp,
}: Props) {
  const { t } = useI18n();
  const [loadedSettings, setLoadedSettings] = useState<ShopSettings | null>(
    null,
  );

  useEffect(() => {
    if (shopSettingsProp !== undefined) {
      return;
    }
    let cancelled = false;
    void settingsApi
      .getShopSettings()
      .then((settings) => {
        if (!cancelled) setLoadedSettings(settings);
      })
      .catch(() => {
        /* preview stays empty without tax settings */
      });
    return () => {
      cancelled = true;
    };
  }, [shopSettingsProp]);

  const shopSettings =
    shopSettingsProp !== undefined ? shopSettingsProp : loadedSettings;
  const currency = shopSettings?.currency ?? "EUR";

  return (
    <section className="flex max-w-xl flex-col gap-3">
      <h2 className="text-sm font-medium">
        {t("repairs.intake.estimate.heading")}
      </h2>
      <p className="text-sm text-muted">{t("repairs.intake.estimate.hint")}</p>

      <RepairEstimateFields
        form={form}
        currency={currency}
        taxRatePercent={shopSettings?.taxRatePercent ?? null}
        disabled={disabled}
      />
    </section>
  );
}
