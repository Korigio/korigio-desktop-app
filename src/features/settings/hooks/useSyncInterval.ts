import { useCallback, useEffect, useState } from "react";
import { settingsApi } from "@/features/settings/api/settingsApi";
import { useI18n } from "@/shared/hooks/useI18n";

export function useSyncInterval() {
  const { t } = useI18n();
  const [value, setValue] = useState("");
  const [saved, setSaved] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    void settingsApi
      .getSyncInterval()
      .then((next) => {
        if (cancelled) {
          return;
        }
        setSaved(next.intervalSeconds);
        setValue(String(next.intervalSeconds));
        setError(null);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(
            err instanceof Error
              ? err.message
              : t("settings.syncInterval.error"),
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [t]);

  const save = useCallback(async () => {
    const trimmed = value.trim();
    if (!/^\d+$/.test(trimmed)) {
      setError(t("settings.syncInterval.error"));
      return;
    }
    const intervalSeconds = Number.parseInt(trimmed, 10);
    if (saved === intervalSeconds) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      const next = await settingsApi.setSyncInterval(intervalSeconds);
      setSaved(next.intervalSeconds);
      setValue(String(next.intervalSeconds));
    } catch (err) {
      setError(
        err instanceof Error ? err.message : t("settings.syncInterval.error"),
      );
    } finally {
      setSaving(false);
    }
  }, [saved, t, value]);

  return {
    value,
    setValue,
    loading,
    saving,
    error,
    save,
  };
}
