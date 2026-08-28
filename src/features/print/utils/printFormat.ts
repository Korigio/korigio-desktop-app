import type { PrintCompany } from "@/features/print/types/printReport";

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
