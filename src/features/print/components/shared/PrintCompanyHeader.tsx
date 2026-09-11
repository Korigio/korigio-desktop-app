import { convertFileSrc } from "@tauri-apps/api/core";
import type { PrintCompany } from "@/features/print/types/printReport";
import { companyDisplayName } from "@/features/print/utils/printFormat";

export type PrintCompanyFieldLabels = {
  taxId: string;
  address: string;
  phone: string;
  email: string;
  website?: string;
};

type Props = {
  company: PrintCompany | null;
  companyLogoAbsolutePath: string | null;
  logoAlt: string;
  appNameFallback: string;
  fieldLabels: PrintCompanyFieldLabels;
};

function contactLines(
  company: PrintCompany,
  fieldLabels: PrintCompanyFieldLabels,
): string[] {
  const lines: string[] = [];
  const address = company.address?.trim();
  const phone = company.phone?.trim();
  const email = company.email?.trim();
  const website = company.website?.trim();
  const taxId = company.taxId?.trim();

  if (address) lines.push(address);
  if (phone) lines.push(phone);
  if (email) lines.push(email);
  if (fieldLabels.website && website) lines.push(website);
  if (taxId) lines.push(`${fieldLabels.taxId} ${taxId}`);
  return lines;
}

export function PrintCompanyHeader({
  company,
  companyLogoAbsolutePath,
  logoAlt,
  appNameFallback,
  fieldLabels,
}: Props) {
  const logoSrc = companyLogoAbsolutePath
    ? convertFileSrc(companyLogoAbsolutePath)
    : null;
  const displayName = company ? companyDisplayName(company) : appNameFallback;
  const showLegalName = Boolean(company && company.legalName !== displayName);

  return (
    <header className="pt-4">
      <div className="flex items-start justify-between gap-6">
        <div className="flex min-w-0 items-start gap-3">
          {logoSrc ? (
            <img
              src={logoSrc}
              alt={logoAlt}
              className="h-16 w-auto max-w-[40%] object-contain"
            />
          ) : null}
          <div className="min-w-0">
            <p className="text-2xl font-semibold tracking-tight">
              {displayName}
            </p>
            {showLegalName && company ? (
              <p className="text-sm text-neutral-700">{company.legalName}</p>
            ) : null}
          </div>
        </div>
        {company ? (
          <div className="shrink-0 text-right text-sm leading-snug">
            {contactLines(company, fieldLabels).map((line) => (
              <p key={line}>{line}</p>
            ))}
          </div>
        ) : null}
      </div>
      <div className="mt-3 border-b border-[#cccccc]" />
    </header>
  );
}
