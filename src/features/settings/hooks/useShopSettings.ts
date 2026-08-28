import { useCallback, useEffect, useState } from "react";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type { ShopSettings } from "@/features/settings/types/shopSettings";
import { useI18n } from "@/shared/hooks/useI18n";

export function useShopSettings() {
  const { t } = useI18n();
  const [settings, setSettings] = useState<ShopSettings | null>(null);
  const [taxRatePercent, setTaxRatePercent] = useState("");
  const [currency, setCurrency] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await settingsApi.getShopSettings();
      setSettings(next);
      setTaxRatePercent(next.taxRatePercent);
      setCurrency(next.currency);
    } catch (err) {
      setSettings(null);
      setError(
        err instanceof Error
          ? err.message
          : t("settings.shop.errors.loadFailed"),
      );
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const save = useCallback(async () => {
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const next = await settingsApi.setShopSettings({
        taxRatePercent,
        currency,
      });
      setSettings(next);
      setTaxRatePercent(next.taxRatePercent);
      setCurrency(next.currency);
      setSuccess(t("settings.shop.saveSuccess"));
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("settings.shop.errors.saveFailed"),
      );
    } finally {
      setSaving(false);
    }
  }, [currency, t, taxRatePercent]);

  return {
    settings,
    taxRatePercent,
    setTaxRatePercent,
    currency,
    setCurrency,
    loading,
    saving,
    error,
    success,
    save,
    reload,
  };
}
