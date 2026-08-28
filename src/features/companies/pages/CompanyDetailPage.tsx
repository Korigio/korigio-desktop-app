import { CompanyContactCard } from "@/features/companies/components/CompanyContactCard";
import { CompanyDetailActions } from "@/features/companies/components/CompanyDetailActions";
import { CompanyDetailStatusPanel } from "@/features/companies/components/CompanyDetailStatusPanel";
import { CompanyIdentityCard } from "@/features/companies/components/CompanyIdentityCard";
import { CompanyLoadState } from "@/features/companies/components/CompanyLoadState";
import { CompanyLogoCard } from "@/features/companies/components/CompanyLogoCard";
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
    <Page className="max-w-5xl">
      <PageHeader
        title={companyLabel(company)}
        description={t("companies.detail.subtitle")}
        actions={
          <CompanyDetailActions
            company={company}
            onCompanyChange={setCompany}
            onBack={() => navigate("/companies")}
          />
        }
      />

      <div className="flex flex-col gap-4">
        <CompanyDetailStatusPanel company={company} />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 lg:gap-6">
          <CompanyIdentityCard company={company} />
          <CompanyContactCard company={company} />
          <CompanyLogoCard
            company={company}
            onCompanyChange={setCompany}
            disabled={isArchived}
          />
        </div>
      </div>
    </Page>
  );
}
