import { CustomerForm } from "@/features/customers/components/CustomerForm";
import { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function CustomerCreatePage() {
  const { t } = useI18n();
  const form = useCustomerForm({ mode: "create" });

  return (
    <Page>
      <PageHeader
        title={t("customers.createTitle")}
        description={t("customers.createSubtitle")}
      />
      <CustomerForm form={form} submitLabel={t("customers.actions.create")} />
    </Page>
  );
}
