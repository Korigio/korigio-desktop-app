import { createContext } from "react";
import type { Locale } from "@/i18n";

export type LocalePreference = "system" | Locale;

export type I18nContextValue = {
  /** Active message catalog. */
  locale: Locale;
  /** Stored preference (`system` or a fixed locale). */
  preference: LocalePreference;
  /** OS locale tag from the backend (best effort). */
  systemLocale: string;
  setPreference: (preference: LocalePreference) => Promise<void>;
  t: (key: string) => string;
};

export const I18nContext = createContext<I18nContextValue | null>(null);
