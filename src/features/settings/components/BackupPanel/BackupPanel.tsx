import { useBackupSettings } from "@/features/settings/hooks/useBackupSettings";
import { Button, Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

function formatSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function BackupPanel() {
  const { t } = useI18n();
  const state = useBackupSettings();

  return (
    <Card
      title={t("settings.backup.title")}
      actions={
        <>
          <Button
            type="button"
            disabled={state.busy}
            onClick={() => void state.createBackup()}
          >
            {state.busy
              ? t("common.saving")
              : t("settings.backup.actions.create")}
          </Button>
          <Button
            type="button"
            variant="secondary"
            disabled={state.busy}
            onClick={() => void state.restoreBackup()}
          >
            {t("settings.backup.actions.restore")}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        {state.error ? (
          <StatusMessage tone="danger">{state.error}</StatusMessage>
        ) : null}
        {state.success ? (
          <StatusMessage tone="success">{state.success}</StatusMessage>
        ) : null}

        <div>
          <h3 className="mb-2 text-sm font-medium">
            {t("settings.backup.localList")}
          </h3>
          {state.loading ? (
            <StatusMessage>{t("common.loading")}</StatusMessage>
          ) : state.items.length === 0 ? (
            <p className="text-sm text-muted">{t("settings.backup.empty")}</p>
          ) : (
            <ul className="flex flex-col gap-2">
              {state.items.map((item) => (
                <li
                  key={item.path}
                  className="border-b border-border py-2 text-sm last:border-b-0"
                >
                  <p className="font-medium">{item.fileName}</p>
                  <p className="text-muted">
                    {t(`settings.backup.kind.${item.kind}`)} · {item.createdAt} ·{" "}
                    {formatSize(item.sizeBytes)}
                  </p>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </Card>
  );
}
