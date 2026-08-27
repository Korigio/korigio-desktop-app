import { BackupPanel } from "@/features/settings/components/BackupPanel";
import { DevSeedPanel } from "@/features/settings/components/DevSeedPanel";
import { LanguagePreferenceField } from "@/features/settings/components/LanguagePreferenceField";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function SettingsPage() {
  const { t } = useI18n();

  return (
    <Page>
      <PageHeader
        title={t("settings.title")}
        description={t("settings.subtitle")}
      />
      <LanguagePreferenceField />
      <BackupPanel />
      <DevSeedPanel />
    </Page>
  );
}
