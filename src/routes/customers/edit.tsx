import { useParams } from "react-router";
import { CustomerEditPage } from "@/features/customers/pages/CustomerEditPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function CustomersEditRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return <p className="p-6 text-sm text-red-700">{t("customers.invalidId")}</p>;
  }
  return <CustomerEditPage customerId={id} />;
}
