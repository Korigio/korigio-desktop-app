import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { Button, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  customer: Customer;
  onCustomerChange: (customer: Customer) => void;
  onBack: () => void;
};

export function CustomerDetailActions({
  customer,
  onCustomerChange,
  onBack,
}: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(customer.archivedAt);

  return (
    <>
      <LinkButton to={`/customers/${customer.id}/edit`} variant="secondary">
        {t("customers.actions.edit")}
      </LinkButton>
      <Button
        type="button"
        variant="secondary"
        onClick={() => {
          void (async () => {
            const next = isArchived
              ? await customersApi.unarchive(customer.id)
              : await customersApi.archive(customer.id);
            onCustomerChange(next);
          })();
        }}
      >
        {isArchived
          ? t("customers.actions.unarchive")
          : t("customers.actions.archive")}
      </Button>
      <Button type="button" variant="secondary" onClick={onBack}>
        {t("customers.backToList")}
      </Button>
    </>
  );
}
