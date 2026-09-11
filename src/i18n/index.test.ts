import { describe, expect, it } from "vitest";
import { defaultLocale, messages, translate } from "./index";

function leafKeys(value: unknown, prefix = ""): string[] {
  if (typeof value === "string") return [prefix];
  if (!value || typeof value !== "object") return [];
  return Object.entries(value).flatMap(([key, child]) =>
    leafKeys(child, prefix ? `${prefix}.${key}` : key),
  );
}

describe("message catalogs", () => {
  it("uses Spanish as the unsupported-locale fallback", () => {
    expect(defaultLocale).toBe("es");
  });

  it("keeps locale leaf keys in parity", () => {
    const expected = leafKeys(messages.es).sort();
    expect(leafKeys(messages.en).sort()).toEqual(expected);
    expect(leafKeys(messages.de).sort()).toEqual(expected);
  });

  it("looks up leaves and returns the key for missing or non-leaf paths", () => {
    expect(translate("en", "common.loading")).not.toBe("common.loading");
    expect(translate("es", "missing.key")).toBe("missing.key");
    expect(translate("de", "common")).toBe("common");
  });
});
