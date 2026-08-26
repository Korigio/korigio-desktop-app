import { CustomerDetailActions } from "@/features/customers/components/CustomerDetailActions";
import { CustomerDetailFields } from "@/features/customers/components/CustomerDetailFields";
import { CustomerLoadState } from "@/features/customers/components/CustomerLoadState";
import { useCustomerDetail } from "@/features/customers/hooks/useCustomerDetail";
import { CustomerDevicesSection } from "@/features/devices/components/CustomerDevicesSection";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { customerId: number };

export function CustomerDetailPage({ customerId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { customer, loading, error, setCustomer } = useCustomerDetail(customerId);

  if (loading) {
    return <CustomerLoadState />;
  }

  if (error || !customer) {
    return <CustomerLoadState message={error ?? t("customers.notFound")} />;
  }

  return (
    <Page>
      <PageHeader
        title={customer.name}
        description={
          customer.archivedAt
            ? t("customers.status.archived")
            : t("customers.status.active")
        }
        actions={
          <CustomerDetailActions
            customer={customer}
            onCustomerChange={setCustomer}
            onBack={() => navigate("/customers")}
          />
        }
      />
      <CustomerDetailFields customer={customer} />
      <CustomerDevicesSection customerId={customer.id} />
    </Page>
  );
}
