import { invoke } from "@/shared/api/invoke";

export type LocalePreferenceDto = "system" | "en" | "es" | "de";

export type LocaleSettingsDto = {
  preference: LocalePreferenceDto;
  systemLocale: string;
  resolvedLocale: string;
};

export const settingsApi = {
  getLocaleSettings(): Promise<LocaleSettingsDto> {
    return invoke<LocaleSettingsDto>("get_locale_settings");
  },
  setLocalePreference(
    preference: LocalePreferenceDto,
  ): Promise<LocaleSettingsDto> {
    return invoke<LocaleSettingsDto>("set_locale_preference", { preference });
  },
};
