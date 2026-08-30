import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  I18nContext,
  type LocalePreference,
} from "@/app/providers/i18n-context";
import { settingsApi, type LocaleSettingsDto } from "@/features/settings/api/settingsApi";
import { defaultLocale, translate, type Locale } from "@/i18n";

function toLocale(value: string): Locale {
  if (value === "en" || value === "de" || value === "es") {
    return value;
  }
  return defaultLocale;
}

function toPreference(value: LocaleSettingsDto["preference"]): LocalePreference {
  if (value === "en" || value === "es" || value === "de" || value === "system") {
    return value;
  }
  return "system";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<Locale>(defaultLocale);
  const [preference, setPreferenceState] = useState<LocalePreference>("system");
  const [systemLocale, setSystemLocale] = useState("");

  useEffect(() => {
    let cancelled = false;
    void settingsApi
      .getLocaleSettings()
      .then((settings) => {
        if (cancelled) {
          return;
        }
        setLocale(toLocale(settings.resolvedLocale));
        setPreferenceState(toPreference(settings.preference));
        setSystemLocale(settings.systemLocale);
      })
      .catch(() => {
        // Browser / unit contexts without Tauri: fall back to navigator.
        if (cancelled) {
          return;
        }
        const tag = navigator.language || defaultLocale;
        const lower = tag.toLowerCase();
        const resolved = lower.startsWith("de")
          ? "de"
          : lower.startsWith("en")
            ? "en"
            : lower.startsWith("es")
              ? "es"
              : defaultLocale;
        setLocale(resolved);
        setPreferenceState("system");
        setSystemLocale(tag);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const setPreference = useCallback(async (next: LocalePreference) => {
    const settings = await settingsApi.setLocalePreference(next);
    setLocale(toLocale(settings.resolvedLocale));
    setPreferenceState(toPreference(settings.preference));
    setSystemLocale(settings.systemLocale);
  }, []);

  const t = useCallback((key: string) => translate(locale, key), [locale]);

  const value = useMemo(
    () => ({ locale, preference, systemLocale, setPreference, t }),
    [locale, preference, systemLocale, setPreference, t],
  );

  return (
    <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
  );
}
