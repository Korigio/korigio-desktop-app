import { open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useState } from "react";
import { backupApi } from "@/features/settings/api/backupApi";
import type { BackupInfo } from "@/features/settings/types/backup";
import { isCommandError } from "@/shared/api/invoke";
import { useI18n } from "@/shared/hooks/useI18n";

function defaultBackupFileName(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  const stamp = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}`;
  return `Korigio-${stamp}.backup`;
}

export function useBackupSettings() {
  const { t } = useI18n();
  const [items, setItems] = useState<BackupInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await backupApi.listLocal();
      setItems(result.items);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("settings.backup.errors.listFailed"),
      );
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const createBackup = useCallback(async () => {
    setBusy(true);
    setError(null);
    setSuccess(null);
    try {
      const destinationPath = await save({
        defaultPath: defaultBackupFileName(),
        filters: [{ name: "Korigio backup", extensions: ["backup"] }],
      });
      const info = await backupApi.create({
        destinationPath: destinationPath ?? null,
      });
      setSuccess(
        t("settings.backup.createSuccess").replace("{path}", info.path),
      );
      await reload();
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("settings.backup.errors.createFailed"),
      );
    } finally {
      setBusy(false);
    }
  }, [reload, t]);

  const restoreBackup = useCallback(async () => {
    setError(null);
    setSuccess(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Korigio backup", extensions: ["backup"] }],
      });
      if (selected === null || Array.isArray(selected)) {
        return;
      }

      const validation = await backupApi.validate(selected);
      if (!validation.valid) {
        setError(
          validation.errors.join(" ") ||
            t("settings.backup.errors.invalidPackage"),
        );
        return;
      }

      const confirmed = window.confirm(t("settings.backup.restoreConfirm"));
      if (!confirmed) {
        return;
      }

      setBusy(true);
      const result = await backupApi.restore(selected);
      setSuccess(
        t("settings.backup.restoreSuccess")
          .replace("{from}", result.restoredFrom)
          .replace("{safety}", result.safetyBackupPath),
      );
      await reload();
    } catch (err) {
      if (isCommandError(err) && err.code === "conflict") {
        setError(t("backup.restore.unsupportedSchema"));
      } else {
        setError(
          err instanceof Error
            ? err.message
            : t("settings.backup.errors.restoreFailed"),
        );
      }
    } finally {
      setBusy(false);
    }
  }, [reload, t]);

  return {
    items,
    loading,
    busy,
    error,
    success,
    createBackup,
    restoreBackup,
    reload,
  };
}
