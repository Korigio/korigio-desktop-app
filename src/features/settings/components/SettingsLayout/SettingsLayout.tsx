import { Outlet, useLocation } from "react-router";
import { SettingsNav } from "@/features/settings/components/SettingsNav";
import { settingsCategoryForPath } from "@/features/settings/constants/settingsCategories";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function SettingsLayout() {
  const { t } = useI18n();
  const { pathname } = useLocation();
  const category = settingsCategoryForPath(pathname);
  const categorySubtitle = t(category?.subtitleKey ?? "settings.subtitle");

  return (
    <Page className="max-w-5xl">
      <PageHeader title={t("settings.title")} description={categorySubtitle} />
      <SettingsNav />
      <Outlet />
    </Page>
  );
}
