export type Company = {
  id: number;
  legalName: string;
  tradeName: string | null;
  taxId: string | null;
  address: string | null;
  phone: string | null;
  email: string | null;
  website: string | null;
  logoPath: string | null;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type CompanyInput = {
  legalName: string;
  tradeName?: string | null;
  taxId?: string | null;
  address?: string | null;
  phone?: string | null;
  email?: string | null;
  website?: string | null;
};

export type CompanyListQuery = {
  query?: string;
  includeArchived?: boolean;
  page?: number;
  pageSize?: number;
};

export type CompanyListResult = {
  items: Company[];
  total: number;
  page: number;
  pageSize: number;
};

export type AttachCompanyLogoInput = {
  companyId: number;
  sourcePath: string;
};

export type ResolveCompanyLogoPathResult = {
  absolutePath: string;
};

/** Prefer trade name when set; otherwise legal name. */
export function companyLabel(company: Company): string {
  const trade = company.tradeName?.trim();
  return trade && trade.length > 0 ? trade : company.legalName;
}
