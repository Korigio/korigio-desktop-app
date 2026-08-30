import { invoke } from "@/shared/api/invoke";
import type {
  AttachCompanyLogoInput,
  Company,
  CompanyInput,
  CompanyListQuery,
  CompanyListResult,
  ResolveCompanyLogoPathResult,
} from "@/features/companies/types/company";

export const companiesApi = {
  list(query: CompanyListQuery = {}): Promise<CompanyListResult> {
    return invoke<CompanyListResult>("list_companies", { query });
  },
  get(id: string): Promise<Company> {
    return invoke<Company>("get_company", { id });
  },
  create(input: CompanyInput): Promise<Company> {
    return invoke<Company>("create_company", { input });
  },
  update(id: string, input: CompanyInput): Promise<Company> {
    return invoke<Company>("update_company", { id, input });
  },
  archive(id: string): Promise<Company> {
    return invoke<Company>("archive_company", { id });
  },
  unarchive(id: string): Promise<Company> {
    return invoke<Company>("unarchive_company", { id });
  },
  setDefault(id: string): Promise<Company> {
    return invoke<Company>("set_default_company", { id });
  },
  attachLogo(input: AttachCompanyLogoInput): Promise<Company> {
    return invoke<Company>("attach_company_logo", { input });
  },
  clearLogo(id: string): Promise<Company> {
    return invoke<Company>("clear_company_logo", { id });
  },
  resolveLogoPath(id: string): Promise<ResolveCompanyLogoPathResult> {
    return invoke<ResolveCompanyLogoPathResult>("resolve_company_logo_path", {
      id,
    });
  },
};
