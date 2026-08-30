import { useAutoBackupSettings } from "@/features/settings/hooks/useAutoBackupSettings";
import {
  AUTO_BACKUP_INTERVALS,
  isAutoBackupInterval,
} from "@/features/settings/types/autoBackup";
import { Button, Card, FormField, SelectField, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function ScheduledBackupSettings() {
  const { t } = useI18n();
  const state = useAutoBackupSettings();

  return (
    <Card title={t("settings.backup.scheduled.title")}>
      <div className="flex flex-col gap-3">
        <p className="text-sm text-muted">
          {t("settings.backup.scheduled.hint")}
        </p>
        {state.loading ? (
          <StatusMessage>{t("common.loading")}</StatusMessage>
        ) : state.settings ? (
          <>
            <FormField
              label={t("settings.backup.scheduled.intervalLabel")}
              htmlFor="settings-scheduled-backup-interval"
            >
              <SelectField
                id="settings-scheduled-backup-interval"
                value={state.settings.interval}
                disabled={state.busy}
                onChange={(event) => {
                  const next = event.target.value;
                  if (!isAutoBackupInterval(next)) {
                    return;
                  }
                  void state.changeInterval(next);
                }}
              >
                {AUTO_BACKUP_INTERVALS.map((interval) => (
                  <option key={interval} value={interval}>
                    {t(`settings.backup.scheduled.intervals.${interval}`)}
                  </option>
                ))}
              </SelectField>
            </FormField>
            <FormField
              label={t("settings.backup.scheduled.folderLabel")}
              htmlFor="settings-scheduled-backup-folder"
            >
              <div className="flex flex-col gap-2">
                <Button
                  id="settings-scheduled-backup-folder"
                  type="button"
                  variant="secondary"
                  disabled={state.busy}
                  onClick={() => void state.pickFolder()}
                >
                  {t("settings.backup.scheduled.chooseFolder")}
                </Button>
                <p className="break-all text-sm text-muted">
                  {state.settings.folderPath ??
                    t("settings.backup.scheduled.folderUnset")}
                </p>
              </div>
            </FormField>
          </>
        ) : null}
        {state.error ? (
          <StatusMessage tone="danger">{state.error}</StatusMessage>
        ) : null}
      </div>
    </Card>
  );
}
