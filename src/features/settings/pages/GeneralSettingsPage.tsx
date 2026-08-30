import { LanguagePreferenceField } from "@/features/settings/components/LanguagePreferenceField";
import { SyncIntervalField } from "@/features/settings/components/SyncIntervalField";
import { ThemePreferenceField } from "@/features/settings/components/ThemePreferenceField";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function GeneralSettingsPage() {
  const { t } = useI18n();

  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <Card title={t("settings.language.label")}>
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">{t("settings.language.hint")}</p>
          <LanguagePreferenceField />
        </div>
      </Card>
      <Card title={t("settings.theme.label")}>
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">{t("settings.theme.hint")}</p>
          <ThemePreferenceField />
        </div>
      </Card>
      <Card title={t("settings.syncInterval.label")}>
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">{t("settings.syncInterval.hint")}</p>
          <p className="text-sm text-muted">
            {t("settings.syncInterval.rangeHint")}
          </p>
          <SyncIntervalField />
        </div>
      </Card>
    </div>
  );
}
