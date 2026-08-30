import { CustomerForm } from "@/features/customers/components/CustomerForm";
import { CustomerLoadState } from "@/features/customers/components/CustomerLoadState";
import { useCustomerDetail } from "@/features/customers/hooks/useCustomerDetail";
import { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import type { Customer } from "@/features/customers/types/customer";
import { Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { customerId: string };

export function CustomerEditPage({ customerId }: Props) {
  const { t } = useI18n();
  const { customer, loading, error } = useCustomerDetail(customerId);

  if (loading) {
    return <CustomerLoadState />;
  }

  if (error || !customer) {
    return <CustomerLoadState message={error ?? t("customers.notFound")} />;
  }

  return <CustomerEditForm customer={customer} />;
}

function CustomerEditForm({ customer }: { customer: Customer }) {
  const { t } = useI18n();
  const form = useCustomerForm({ mode: "edit", customer });
  const isArchived = Boolean(customer.archivedAt);

  return (
    <Page>
      <PageHeader title={t("customers.editTitle")} description={customer.name} />
      <CustomerForm
        form={form}
        submitLabel={t("customers.actions.save")}
        disabled={isArchived}
      />
      {isArchived ? (
        <StatusMessage>{t("customers.archivedEditHint")}</StatusMessage>
      ) : null}
    </Page>
  );
}
