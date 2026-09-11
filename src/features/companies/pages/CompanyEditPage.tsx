import { CompanyForm } from "@/features/companies/components/CompanyForm";
import { CompanyLoadState } from "@/features/companies/components/CompanyLoadState";
import { CompanyLogoCard } from "@/features/companies/components/CompanyLogoCard";
import { useCompanyDetail } from "@/features/companies/hooks/useCompanyDetail";
import { useCompanyForm } from "@/features/companies/hooks/useCompanyForm";
import { companyLabel, type Company } from "@/features/companies/types/company";
import { Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { companyId: string };

export function CompanyEditPage({ companyId }: Props) {
  const { t } = useI18n();
  const { company, loading, error, setCompany } = useCompanyDetail(companyId);

  if (loading) {
    return <CompanyLoadState />;
  }

  if (error || !company) {
    return <CompanyLoadState message={error ?? t("companies.notFound")} />;
  }

  return <CompanyEditForm company={company} onCompanyChange={setCompany} />;
}

function CompanyEditForm({
  company,
  onCompanyChange,
}: {
  company: Company;
  onCompanyChange: (company: Company) => void;
}) {
  const { t } = useI18n();
  const form = useCompanyForm({ mode: "edit", company });
  const isArchived = Boolean(company.archivedAt);

  return (
    <Page>
      <PageHeader
        title={t("companies.editTitle")}
        description={companyLabel(company)}
      />
      <CompanyForm
        form={form}
        submitLabel={t("companies.actions.save")}
        disabled={isArchived}
      />
      <CompanyLogoCard
        company={company}
        onCompanyChange={onCompanyChange}
        disabled={isArchived}
      />
      {isArchived ? (
        <StatusMessage>{t("companies.archivedEditHint")}</StatusMessage>
      ) : null}
    </Page>
  );
}
