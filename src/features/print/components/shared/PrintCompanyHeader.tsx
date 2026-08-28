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

  return (
    <header className="flex items-start justify-between gap-4 border-b border-black pb-3">
      <div className="min-w-0 flex-1">
        {company ? (
          <>
            <p className="text-2xl font-semibold tracking-tight">
              {companyDisplayName(company)}
            </p>
            {company.legalName !== companyDisplayName(company) ? (
              <p className="text-sm">{company.legalName}</p>
            ) : null}
            <dl className="mt-2 grid grid-cols-[6rem_1fr] gap-x-3 gap-y-0.5 text-sm">
              {company.taxId ? (
                <>
                  <dt>{fieldLabels.taxId}</dt>
                  <dd>{company.taxId}</dd>
                </>
              ) : null}
              {company.address ? (
                <>
                  <dt>{fieldLabels.address}</dt>
                  <dd>{company.address}</dd>
                </>
              ) : null}
              {company.phone ? (
                <>
                  <dt>{fieldLabels.phone}</dt>
                  <dd>{company.phone}</dd>
                </>
              ) : null}
              {company.email ? (
                <>
                  <dt>{fieldLabels.email}</dt>
                  <dd>{company.email}</dd>
                </>
              ) : null}
              {fieldLabels.website && company.website ? (
                <>
                  <dt>{fieldLabels.website}</dt>
                  <dd>{company.website}</dd>
                </>
              ) : null}
            </dl>
          </>
        ) : (
          <p className="text-2xl font-semibold tracking-tight">{appNameFallback}</p>
        )}
      </div>
      {logoSrc ? (
        <img
          src={logoSrc}
          alt={logoAlt}
          className="h-16 w-auto max-w-[40%] object-contain"
        />
      ) : null}
    </header>
  );
}
