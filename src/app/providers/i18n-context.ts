import { createContext } from "react";
import type { Locale } from "@/i18n";

export type I18nContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
};

export const I18nContext = createContext<I18nContextValue | null>(null);
