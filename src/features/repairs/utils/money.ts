/** Integer-cent tax math mirroring Rust `tax_cents_from_base` (round half up). */

export type EstimatePreview = {
  baseCents: number;
  taxRateBps: number;
  taxCents: number;
  grossCents: number;
};

/** Parse tax percent string (0–100, ≤2 decimals) to basis points. */
export function parseTaxRatePercentToBps(raw: string): number | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;

  const [intPart, fracPart = ""] = trimmed.split(".");
  if (
    !intPart ||
    !/^\d+$/.test(intPart) ||
    (fracPart.length > 0 && !/^\d+$/.test(fracPart)) ||
    fracPart.length > 2
  ) {
    return null;
  }

  const whole = Number(intPart);
  const fracDigits = fracPart.length;
  const fracValue = fracPart ? Number(fracPart) : 0;
  const fracBps =
    fracDigits === 0 ? 0 : fracDigits === 1 ? fracValue * 10 : fracValue;
  const bps = whole * 100 + fracBps;
  if (bps < 0 || bps > 10_000) return null;
  return bps;
}

/** Round half up: `(base * bps + 5000) / 10000` for non-negative integers. */
export function taxCentsFromBase(
  baseCents: number,
  taxRateBps: number,
): number | null {
  if (
    !Number.isInteger(baseCents) ||
    !Number.isInteger(taxRateBps) ||
    baseCents < 0 ||
    taxRateBps < 0
  ) {
    return null;
  }
  const product = baseCents * taxRateBps;
  if (!Number.isSafeInteger(product)) return null;
  return Math.floor((product + 5_000) / 10_000);
}

export function previewEstimate(
  baseCents: number,
  taxRatePercent: string,
): EstimatePreview | null {
  const taxRateBps = parseTaxRatePercentToBps(taxRatePercent);
  if (taxRateBps === null) return null;
  const taxCents = taxCentsFromBase(baseCents, taxRateBps);
  if (taxCents === null) return null;
  const grossCents = baseCents + taxCents;
  if (!Number.isSafeInteger(grossCents)) return null;
  return { baseCents, taxRateBps, taxCents, grossCents };
}

/** Parse a major-unit amount (`"100"`, `"100.5"`, `"100.50"`) to integer cents. */
export function parseMajorToCents(raw: string): number | null {
  const trimmed = raw.trim().replace(",", ".");
  if (!trimmed) return null;
  if (!/^\d+(\.\d{1,2})?$/.test(trimmed)) return null;
  const [wholePart, fracPart = ""] = trimmed.split(".");
  const whole = Number(wholePart);
  const fracPadded = `${fracPart}00`.slice(0, 2);
  const cents = whole * 100 + Number(fracPadded);
  if (!Number.isSafeInteger(cents) || cents < 0) return null;
  return cents;
}

export function centsToMajorInput(cents: number): string {
  const negative = cents < 0;
  const abs = Math.abs(cents);
  const whole = Math.floor(abs / 100);
  const frac = abs % 100;
  const body =
    frac === 0 ? String(whole) : `${whole}.${String(frac).padStart(2, "0")}`;
  return negative ? `-${body}` : body;
}

export function formatMoneyCents(
  cents: number,
  currency: string,
  locale?: string,
): string {
  try {
    return new Intl.NumberFormat(locale, {
      style: "currency",
      currency: currency || "EUR",
    }).format(cents / 100);
  } catch {
    return `${(cents / 100).toFixed(2)} ${currency}`;
  }
}

export function formatTaxRateBps(bps: number): string {
  if (bps % 100 === 0) return String(bps / 100);
  if (bps % 10 === 0) return (bps / 100).toFixed(1);
  return (bps / 100).toFixed(2);
}
