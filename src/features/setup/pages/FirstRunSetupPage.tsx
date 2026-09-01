import { FirstRunSetupForm } from "@/features/setup/components/FirstRunSetupForm";
import { useFirstRunForm } from "@/features/setup/hooks/useFirstRunForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function FirstRunSetupPage() {
  const { t } = useI18n();
  const setup = useFirstRunForm();

  return (
    <Page>
      <PageHeader title={t("setup.title")} description={t("setup.subtitle")} />
      <FirstRunSetupForm setup={setup} />
    </Page>
  );
}
