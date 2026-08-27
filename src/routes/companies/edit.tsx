import { useParams } from "react-router";
import { CompanyEditPage } from "@/features/companies/pages/CompanyEditPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function CompaniesEditRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return (
      <p className="p-6 text-sm text-red-700">{t("companies.invalidId")}</p>
    );
  }
  return <CompanyEditPage companyId={id} />;
}
