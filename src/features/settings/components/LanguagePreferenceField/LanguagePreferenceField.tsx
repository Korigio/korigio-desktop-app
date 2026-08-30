import { useState } from "react";
import type { LocalePreference } from "@/app/providers/i18n-context";
import { FormField, SelectField, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function LanguagePreferenceField() {
  const { t, preference, systemLocale, setPreference } = useI18n();
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  return (
    <div className="flex flex-col gap-2">
      <FormField label={t("settings.language.label")} htmlFor="settings-language">
        <SelectField
          id="settings-language"
          value={preference}
          disabled={saving}
          onChange={(event) => {
            const next = event.target.value as LocalePreference;
            setSaving(true);
            setError(null);
            void setPreference(next)
              .catch((err: unknown) => {
                setError(
                  err instanceof Error
                    ? err.message
                    : t("settings.language.error"),
                );
              })
              .finally(() => setSaving(false));
          }}
        >
          <option value="system">
            {t("settings.language.system")}
            {systemLocale ? ` (${systemLocale})` : ""}
          </option>
          <option value="de">{t("settings.language.de")}</option>
          <option value="es">{t("settings.language.es")}</option>
          <option value="en">{t("settings.language.en")}</option>
        </SelectField>
      </FormField>
      {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
    </div>
  );
}
