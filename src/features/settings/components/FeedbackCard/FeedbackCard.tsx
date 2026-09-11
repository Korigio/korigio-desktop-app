import { useState } from "react";
import { updatesApi } from "@/features/settings/api/updatesApi";
import { FEEDBACK_MAILTO_ADDRESS } from "@/features/settings/constants/updates";
import { useAppUpdate } from "@/features/settings/hooks/app-update-context";
import { Button, Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function FeedbackCard() {
  const { t } = useI18n();
  const { currentVersion } = useAppUpdate();
  const [error, setError] = useState<string | null>(null);
  const version = currentVersion ?? "";

  async function openMail() {
    setError(null);
    const subject = t("settings.feedback.mailSubject").replace(
      "{version}",
      version,
    );
    const body = t("settings.feedback.mailBody").replace("{version}", version);
    const url = `mailto:${FEEDBACK_MAILTO_ADDRESS}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
    try {
      await updatesApi.openExternalUrl(url);
    } catch (err) {
      if (err instanceof Error && err.message) {
        setError(err.message);
      }
    }
  }

  return (
    <Card title={t("settings.feedback.title")}>
      <div className="flex flex-col gap-3">
        <p className="text-sm text-muted">{t("settings.feedback.body")}</p>
        <p className="text-sm">
          <span className="text-muted">
            {t("settings.feedback.emailLabel")}:{" "}
          </span>
          <button
            type="button"
            className="text-primary underline underline-offset-2"
            onClick={() => void openMail()}
          >
            {t("settings.feedback.email")}
          </button>
        </p>
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        <div>
          <Button type="button" onClick={() => void openMail()}>
            {t("settings.feedback.openMail")}
          </Button>
        </div>
      </div>
    </Card>
  );
}
