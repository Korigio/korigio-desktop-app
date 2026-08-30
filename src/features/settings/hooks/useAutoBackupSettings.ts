import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type {
  AutoBackupInterval,
  AutoBackupSettings,
} from "@/features/settings/types/autoBackup";
import { useI18n } from "@/shared/hooks/useI18n";

export function useAutoBackupSettings() {
  const { t } = useI18n();
  const [settings, setSettings] = useState<AutoBackupSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await settingsApi.getAutoBackupSettings();
      setSettings(next);
    } catch (err) {
      setSettings(null);
      setError(
        err instanceof Error
          ? err.message
          : t("settings.backup.scheduled.errors.loadFailed"),
      );
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const save = useCallback(
    async (input: AutoBackupSettings) => {
      setBusy(true);
      setError(null);
      try {
        const next = await settingsApi.setAutoBackupSettings(input);
        setSettings(next);
      } catch (err) {
        setError(
          err instanceof Error
            ? err.message
            : t("settings.backup.scheduled.errors.saveFailed"),
        );
      } finally {
        setBusy(false);
      }
    },
    [t],
  );

  const changeInterval = useCallback(
    async (interval: AutoBackupInterval) => {
      if (!settings) {
        return;
      }
      if (interval !== "never" && !settings.folderPath) {
        setError(t("settings.backup.scheduled.errors.saveFailed"));
        return;
      }
      await save({ interval, folderPath: settings.folderPath });
    },
    [save, settings, t],
  );

  const pickFolder = useCallback(async () => {
    if (!settings) {
      return;
    }
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected === null || Array.isArray(selected)) {
      return;
    }
    await save({ interval: settings.interval, folderPath: selected });
  }, [save, settings]);

  return {
    settings,
    loading,
    busy,
    error,
    changeInterval,
    pickFolder,
  };
}
