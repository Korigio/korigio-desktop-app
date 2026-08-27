import { CompanyForm } from "@/features/companies/components/CompanyForm";
import { useCompanyForm } from "@/features/companies/hooks/useCompanyForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function CompanyCreatePage() {
  const { t } = useI18n();
  const form = useCompanyForm({ mode: "create" });

  return (
    <Page>
      <PageHeader
        title={t("companies.createTitle")}
        description={t("companies.createSubtitle")}
      />
      <CompanyForm form={form} submitLabel={t("companies.actions.create")} />
    </Page>
  );
}
