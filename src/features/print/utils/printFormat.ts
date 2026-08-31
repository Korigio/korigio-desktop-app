import type { PrintCompany } from "@/features/print/types/printReport";
import type { Locale } from "@/i18n";

/** Printable fallback when a text field is empty. */
export function printFieldValue(
  value: string | null | undefined,
  empty: string,
): string {
  const trimmed = value?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : empty;
}

export function companyDisplayName(company: PrintCompany): string {
  const trade = company.tradeName?.trim();
  return trade && trade.length > 0 ? trade : company.legalName;
}

const DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/;

function parsePrintDate(iso: string): Date | null {
  const dateOnly = DATE_ONLY.exec(iso);
  if (dateOnly) {
    const year = Number(dateOnly[1]);
    const month = Number(dateOnly[2]);
    const day = Number(dateOnly[3]);
    const date = new Date(year, month - 1, day);
    if (
      date.getFullYear() !== year ||
      date.getMonth() !== month - 1 ||
      date.getDate() !== day
    ) {
      return null;
    }
    return date;
  }

  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? null : date;
}

/** Long localized date for letters. Invalid or missing values return `empty`. */
export function formatPrintDate(
  iso: string | null | undefined,
  locale: Locale,
  empty: string,
): string {
  const trimmed = iso?.trim();
  if (!trimmed) return empty;

  const date = parsePrintDate(trimmed);
  if (!date) return empty;

  try {
    return new Intl.DateTimeFormat(locale, { dateStyle: "long" }).format(date);
  } catch {
    return empty;
  }
}

/** Compact unlabeled footer: address · phone · email (skips empty parts). */
export function printFooterLine(company: PrintCompany | null | undefined): string {
  if (!company) return "";
  return [company.address, company.phone, company.email]
    .map((part) => part?.trim())
    .filter((part): part is string => Boolean(part && part.length > 0))
    .join(" · ");
}
