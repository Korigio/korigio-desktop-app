export type CurrencyOption = {
  code: string;
  symbol: string;
};

/** Curated list of commonly used shop currencies. */
export const CURRENCIES: CurrencyOption[] = [
  { code: "EUR", symbol: "€" },
  { code: "USD", symbol: "$" },
  { code: "GBP", symbol: "£" },
  { code: "CHF", symbol: "CHF" },
  { code: "PLN", symbol: "zł" },
  { code: "CZK", symbol: "Kč" },
  { code: "SEK", symbol: "kr" },
  { code: "NOK", symbol: "kr" },
  { code: "DKK", symbol: "kr" },
  { code: "HUF", symbol: "Ft" },
  { code: "CAD", symbol: "C$" },
  { code: "AUD", symbol: "A$" },
  { code: "JPY", symbol: "¥" },
  { code: "CNY", symbol: "¥" },
  { code: "INR", symbol: "₹" },
];

export function currencyOptionLabel(c: CurrencyOption): string {
  return `${c.code} (${c.symbol})`;
}

/**
 * Options for a currency select. If `currentCode` is set and not in the
 * curated list, prepends a fallback so the existing value stays selectable.
 */
export function currencySelectOptions(currentCode: string): CurrencyOption[] {
  const trimmed = currentCode.trim();
  if (!trimmed) {
    return CURRENCIES;
  }
  const code = trimmed.toUpperCase();
  if (CURRENCIES.some((c) => c.code === code)) {
    return CURRENCIES;
  }
  return [{ code, symbol: code }, ...CURRENCIES];
}
