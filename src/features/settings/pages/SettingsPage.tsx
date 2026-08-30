import { BackupPanel } from "@/features/settings/components/BackupPanel";
import { DevSeedPanel } from "@/features/settings/components/DevSeedPanel";
import { LanguagePreferenceField } from "@/features/settings/components/LanguagePreferenceField";
import { ShopSettingsFields } from "@/features/settings/components/ShopSettingsFields";
import { SyncIntervalField } from "@/features/settings/components/SyncIntervalField";
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
      <div className="flex flex-col gap-8 lg:flex-row lg:items-start lg:gap-12">
        <div className="flex flex-col gap-6">
          <LanguagePreferenceField />
          <SyncIntervalField />
        </div>
        <ShopSettingsFields />
      </div>
      <BackupPanel />
      <DevSeedPanel />
    </Page>
  );
}
