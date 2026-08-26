import en from "./en.json";
import es from "./es.json";
import de from "./de.json";

export type Locale = "en" | "es" | "de";

export const defaultLocale: Locale = "es";

export const messages = {
  en,
  es,
  de,
} as const;

export type MessageTree = typeof es;

export function translate(
  locale: Locale,
  key: string,
): string {
  const parts = key.split(".");
  let current: unknown = messages[locale];

  for (const part of parts) {
    if (
      typeof current !== "object" ||
      current === null ||
      !(part in current)
    ) {
      return key;
    }
    current = (current as Record<string, unknown>)[part];
  }

  return typeof current === "string" ? current : key;
}
