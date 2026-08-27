import { useParams } from "react-router";
import { CustomerDetailPage } from "@/features/customers/pages/CustomerDetailPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function CustomersDetailRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return <p className="p-6 text-sm text-red-700">{t("customers.invalidId")}</p>;
  }
  return <CustomerDetailPage customerId={id} />;
}
