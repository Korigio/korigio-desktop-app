import { useState } from "react";
import type { ThemePreference } from "@/features/settings/api/settingsApi";
import { FormField, SelectField, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useTheme } from "@/shared/hooks/useTheme";

export function ThemePreferenceField() {
  const { t } = useI18n();
  const { preference, setPreference } = useTheme();
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  return (
    <div className="flex flex-col gap-2">
      <FormField label={t("settings.theme.label")} htmlFor="settings-theme">
        <SelectField
          id="settings-theme"
          value={preference}
          disabled={saving}
          onChange={(event) => {
            const next = event.target.value as ThemePreference;
            setSaving(true);
            setError(null);
            void setPreference(next)
              .catch((err: unknown) => {
                setError(
                  err instanceof Error
                    ? err.message
                    : t("settings.theme.error"),
                );
              })
              .finally(() => setSaving(false));
          }}
        >
          <option value="system">{t("settings.theme.system")}</option>
          <option value="light">{t("settings.theme.light")}</option>
          <option value="dark">{t("settings.theme.dark")}</option>
        </SelectField>
      </FormField>
      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
    </div>
  );
}
