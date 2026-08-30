import { invoke } from "@/shared/api/invoke";
import type {
  ShopSettings,
  ShopSettingsInput,
} from "@/features/settings/types/shopSettings";
import type { SyncIntervalSettings } from "@/features/settings/types/syncInterval";

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
  getShopSettings(): Promise<ShopSettings> {
    return invoke<ShopSettings>("get_shop_settings");
  },
  setShopSettings(input: ShopSettingsInput): Promise<ShopSettings> {
    return invoke<ShopSettings>("set_shop_settings", { input });
  },
  getSyncInterval(): Promise<SyncIntervalSettings> {
    return invoke<SyncIntervalSettings>("get_sync_interval");
  },
  setSyncInterval(intervalSeconds: number): Promise<SyncIntervalSettings> {
    return invoke<SyncIntervalSettings>("set_sync_interval", {
      intervalSeconds,
    });
  },
};
