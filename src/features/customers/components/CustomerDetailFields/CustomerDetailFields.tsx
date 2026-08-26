import type { Customer } from "@/features/customers/types/customer";
import { DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  customer: Customer;
};

export function CustomerDetailFields({ customer }: Props) {
  const { t } = useI18n();

  return (
    <DefinitionList
      items={[
        { label: t("customers.fields.phone"), value: customer.phone ?? EMPTY },
        { label: t("customers.fields.email"), value: customer.email ?? EMPTY },
        {
          label: t("customers.fields.address"),
          value: customer.address ?? EMPTY,
        },
        { label: t("customers.fields.notes"), value: customer.notes ?? EMPTY },
      ]}
    />
  );
}
