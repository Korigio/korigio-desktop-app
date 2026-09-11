import type { LocalePreference } from "@/app/providers/i18n-context";
import type { LocaleSettingsDto } from "@/features/settings/api/settingsApi";
import { defaultLocale, type Locale } from "@/i18n";

export function resolveLocale(value: string): Locale {
  return value === "en" || value === "de" || value === "es"
    ? value
    : defaultLocale;
}

export function resolveLocalePreference(
  value: LocaleSettingsDto["preference"],
): LocalePreference {
  return value === "en" ||
    value === "es" ||
    value === "de" ||
    value === "system"
    ? value
    : "system";
}

export function resolveSystemLocale(tag: string): Locale {
  const lower = tag.toLowerCase();
  if (lower.startsWith("de")) return "de";
  if (lower.startsWith("en")) return "en";
  if (lower.startsWith("es")) return "es";
  return defaultLocale;
}
