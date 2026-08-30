export type SettingsCategory = {
  path: string;
  labelKey: string;
  subtitleKey: string;
};

export const settingsCategories: SettingsCategory[] = [
  {
    path: "/settings/general",
    labelKey: "settings.nav.general",
    subtitleKey: "settings.subtitle",
  },
  {
    path: "/settings/shop",
    labelKey: "settings.nav.shop",
    subtitleKey: "settings.shop.subtitle",
  },
  {
    path: "/settings/backup",
    labelKey: "settings.nav.backup",
    subtitleKey: "settings.backup.subtitle",
  },
];

export function settingsCategoryForPath(
  pathname: string,
): SettingsCategory | undefined {
  return settingsCategories.find((category) => category.path === pathname);
}
