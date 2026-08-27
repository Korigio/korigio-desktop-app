import { CompanyDetailActions } from "@/features/companies/components/CompanyDetailActions";
import { CompanyDetailFields } from "@/features/companies/components/CompanyDetailFields";
import { CompanyLoadState } from "@/features/companies/components/CompanyLoadState";
import { CompanyLogoSection } from "@/features/companies/components/CompanyLogoSection";
import { useCompanyDetail } from "@/features/companies/hooks/useCompanyDetail";
import { companyLabel } from "@/features/companies/types/company";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { companyId: number };

export function CompanyDetailPage({ companyId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { company, loading, error, setCompany } = useCompanyDetail(companyId);

  if (loading) {
    return <CompanyLoadState />;
  }

  if (error || !company) {
    return <CompanyLoadState message={error ?? t("companies.notFound")} />;
  }

  const isArchived = Boolean(company.archivedAt);

  return (
    <Page>
      <PageHeader
        title={companyLabel(company)}
        description={
          isArchived
            ? t("companies.status.archived")
            : company.isDefault
              ? t("companies.status.default")
              : t("companies.status.active")
        }
        actions={
          <CompanyDetailActions
            company={company}
            onCompanyChange={setCompany}
            onBack={() => navigate("/companies")}
          />
        }
      />
      <CompanyDetailFields company={company} />
      <CompanyLogoSection
        company={company}
        onCompanyChange={setCompany}
        disabled={isArchived}
      />
    </Page>
  );
}
