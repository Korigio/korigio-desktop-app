import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { updatesApi } from "@/features/settings/api/updatesApi";
import {
  AppUpdateContext,
  type AppUpdateContextValue,
} from "@/features/settings/hooks/app-update-context";
import type { UpdateCheck } from "@/features/settings/types/updates";
import { isCommandError } from "@/shared/api/invoke";
import { useI18n } from "@/shared/hooks/useI18n";

export { useAppUpdate } from "@/features/settings/hooks/app-update-context";

export function AppUpdateProvider({ children }: { children: ReactNode }) {
  const { t } = useI18n();
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [checking, setChecking] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const applyCheck = useCallback((result: UpdateCheck) => {
    setCurrentVersion(result.currentVersion);
    setLatestVersion(result.latestVersion);
    setUpdateAvailable(result.updateAvailable);
    setDownloadUrl(result.downloadUrl);
  }, []);

  const checkErrorMessage = useCallback(
    (err: unknown) => {
      if (isCommandError(err) && err.code === "network") {
        return t("settings.updates.checkFailed");
      }
      if (err instanceof Error && err.message) {
        return err.message;
      }
      return t("settings.updates.checkFailed");
    },
    [t],
  );

  useEffect(() => {
    let cancelled = false;

    async function load() {
      setChecking(true);
      try {
        const version = await updatesApi.getAppVersion();
        if (!cancelled) {
          setCurrentVersion(version.version);
        }
      } catch {
        // Keep going; version stays unset if this command fails.
      }

      try {
        const result = await updatesApi.checkAppUpdate();
        if (!cancelled) {
          applyCheck(result);
          setError(null);
        }
      } catch {
        if (!cancelled) {
          setUpdateAvailable(false);
        }
      } finally {
        if (!cancelled) {
          setChecking(false);
        }
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, [applyCheck]);

  const checkAgain = useCallback(async () => {
    setChecking(true);
    setError(null);
    try {
      const result = await updatesApi.checkAppUpdate();
      applyCheck(result);
      setError(null);
    } catch (err) {
      setError(checkErrorMessage(err));
    } finally {
      setChecking(false);
    }
  }, [applyCheck, checkErrorMessage]);

  const value = useMemo<AppUpdateContextValue>(
    () => ({
      updateAvailable,
      currentVersion,
      latestVersion,
      downloadUrl,
      checking,
      error,
      checkAgain,
    }),
    [
      updateAvailable,
      currentVersion,
      latestVersion,
      downloadUrl,
      checking,
      error,
      checkAgain,
    ],
  );

  return (
    <AppUpdateContext.Provider value={value}>
      {children}
    </AppUpdateContext.Provider>
  );
}
