import { describe, expect, it } from "vitest";
import {
  resolveLocale,
  resolveLocalePreference,
  resolveSystemLocale,
} from "./locale";

describe("locale resolution", () => {
  it.each([
    ["de-DE", "de"],
    ["en-US", "en"],
    ["es-MX", "es"],
    ["fr-FR", "es"],
    ["", "es"],
  ])("maps system locale %s to %s", (tag, expected) =>
    expect(resolveSystemLocale(tag)).toBe(expected),
  );

  it("falls back invalid backend values without changing valid preferences", () => {
    expect(resolveLocale("xx")).toBe("es");
    expect(resolveLocale("de")).toBe("de");
    expect(resolveLocalePreference("system")).toBe("system");
    expect(resolveLocalePreference("invalid" as never)).toBe("system");
  });
});
