import { useSyncInterval } from "@/features/settings/hooks/useSyncInterval";
import { FormField, StatusMessage, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function SyncIntervalField() {
  const { t } = useI18n();
  const interval = useSyncInterval();

  if (interval.loading) {
    return <StatusMessage>{t("common.loading")}</StatusMessage>;
  }

  return (
    <div className="flex flex-col gap-2">
      <FormField
        label={t("settings.syncInterval.label")}
        htmlFor="settings-sync-interval"
      >
        <div className="flex items-center gap-2">
          <TextField
            id="settings-sync-interval"
            type="number"
            inputMode="numeric"
            min={2}
            max={60}
            step={1}
            value={interval.value}
            disabled={interval.saving}
            onChange={(event) => interval.setValue(event.target.value)}
            onBlur={() => void interval.save()}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void interval.save();
              }
            }}
          />
          <span className="text-sm text-muted">
            {t("settings.syncInterval.unit")}
          </span>
        </div>
      </FormField>
      {interval.error ? (
        <StatusMessage tone="danger">{interval.error}</StatusMessage>
      ) : null}
    </div>
  );
}
