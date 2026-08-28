import type { Customer } from "@/features/customers/types/customer";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  customer: Customer;
};

export function CustomerContactCard({ customer }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("customers.detail.sections.contact")}>
      <DefinitionList
        items={[
          { label: t("customers.fields.phone"), value: customer.phone ?? EMPTY },
          { label: t("customers.fields.email"), value: customer.email ?? EMPTY },
          {
            label: t("customers.fields.address"),
            value: customer.address ?? EMPTY,
          },
        ]}
      />
    </Card>
  );
}
