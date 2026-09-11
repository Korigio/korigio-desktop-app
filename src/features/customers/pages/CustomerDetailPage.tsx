import { CustomerContactCard } from "@/features/customers/components/CustomerContactCard";
import { CustomerDetailActions } from "@/features/customers/components/CustomerDetailActions";
import { CustomerDetailStatusPanel } from "@/features/customers/components/CustomerDetailStatusPanel";
import { CustomerLoadState } from "@/features/customers/components/CustomerLoadState";
import { CustomerNotesCard } from "@/features/customers/components/CustomerNotesCard";
import { useCustomerDetail } from "@/features/customers/hooks/useCustomerDetail";
import { CustomerDevicesSection } from "@/features/devices/components/CustomerDevicesSection";
import { CustomerRepairsSection } from "@/features/repairs/components/CustomerRepairsSection";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { customerId: string };

export function CustomerDetailPage({ customerId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { customer, loading, error, setCustomer } =
    useCustomerDetail(customerId);

  if (loading) {
    return <CustomerLoadState />;
  }

  if (error || !customer) {
    return <CustomerLoadState message={error ?? t("customers.notFound")} />;
  }

  return (
    <Page className="max-w-5xl">
      <PageHeader
        title={customer.name}
        description={t("customers.detail.subtitle")}
        actions={
          <CustomerDetailActions
            customer={customer}
            onCustomerChange={setCustomer}
            onBack={() => navigate("/customers")}
          />
        }
      />

      <div className="flex flex-col gap-4">
        <CustomerDetailStatusPanel customer={customer} />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 lg:gap-6">
          <CustomerContactCard customer={customer} />
          <CustomerNotesCard customer={customer} />
        </div>

        <CustomerDevicesSection customerId={customer.id} />
        <CustomerRepairsSection customerId={customer.id} />
      </div>
    </Page>
  );
}
