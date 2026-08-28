import type { Customer } from "@/features/customers/types/customer";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  customer: Customer;
};

export function CustomerNotesCard({ customer }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("customers.detail.sections.notes")}>
      <DefinitionList
        items={[
          { label: t("customers.fields.notes"), value: customer.notes ?? EMPTY },
        ]}
      />
    </Card>
  );
}
