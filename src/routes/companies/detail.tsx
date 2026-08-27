import { useParams } from "react-router";
import { CompanyDetailPage } from "@/features/companies/pages/CompanyDetailPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function CompaniesDetailRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return (
      <p className="p-6 text-sm text-red-700">{t("companies.invalidId")}</p>
    );
  }
  return <CompanyDetailPage companyId={id} />;
}
