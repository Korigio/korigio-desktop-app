import { useState } from "react";
import { updatesApi } from "@/features/settings/api/updatesApi";
import { DOWNLOAD_PAGE_URL } from "@/features/settings/constants/updates";
import { useAppUpdate } from "@/features/settings/hooks/app-update-context";
import { Button, Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function UpdatesCard() {
  const { t } = useI18n();
  const {
    updateAvailable,
    currentVersion,
    latestVersion,
    downloadUrl,
    checking,
    error,
    checkAgain,
  } = useAppUpdate();
  const [actionError, setActionError] = useState<string | null>(null);

  const displayError = actionError ?? error;
  const upToDate =
    !checking && !error && !updateAvailable && latestVersion !== null;

  async function openUrl(url: string) {
    setActionError(null);
    try {
      await updatesApi.openExternalUrl(url);
    } catch (err) {
      if (err instanceof Error && err.message) {
        setActionError(err.message);
      }
    }
  }

  return (
    <Card title={t("settings.updates.title")}>
      <div className="flex flex-col gap-3">
        <p className="text-sm text-muted">{t("settings.updates.hint")}</p>
        <p className="text-sm text-muted">
          {t("settings.updates.offlineHint")}
        </p>
        <p className="text-sm">
          {t("settings.updates.currentVersion").replace(
            "{version}",
            currentVersion ?? "",
          )}
        </p>
        {latestVersion ? (
          <p className="text-sm">
            {t("settings.updates.latestVersion").replace(
              "{version}",
              latestVersion,
            )}
          </p>
        ) : null}
        {checking ? (
          <StatusMessage>{t("settings.updates.checking")}</StatusMessage>
        ) : null}
        {updateAvailable ? (
          <StatusMessage>{t("settings.updates.available")}</StatusMessage>
        ) : null}
        {updateAvailable && !downloadUrl ? (
          <StatusMessage>{t("settings.updates.noInstaller")}</StatusMessage>
        ) : null}
        {upToDate ? (
          <StatusMessage>{t("settings.updates.upToDate")}</StatusMessage>
        ) : null}
        {displayError ? (
          <StatusMessage tone="danger">{displayError}</StatusMessage>
        ) : null}
        <div className="flex flex-wrap gap-2">
          {updateAvailable && downloadUrl ? (
            <Button
              type="button"
              disabled={checking}
              onClick={() => void openUrl(downloadUrl)}
            >
              {t("settings.updates.download")}
            </Button>
          ) : null}
          {updateAvailable && !downloadUrl ? (
            <Button
              type="button"
              variant="secondary"
              disabled={checking}
              onClick={() => void openUrl(DOWNLOAD_PAGE_URL)}
            >
              {t("settings.updates.openDownloadPage")}
            </Button>
          ) : null}
          <Button
            type="button"
            variant="secondary"
            disabled={checking}
            onClick={() => {
              setActionError(null);
              void checkAgain();
            }}
          >
            {checking
              ? t("settings.updates.checking")
              : t("settings.updates.checkAgain")}
          </Button>
        </div>
      </div>
    </Card>
  );
}
