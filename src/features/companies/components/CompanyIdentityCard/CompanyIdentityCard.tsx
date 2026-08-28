import type { Company } from "@/features/companies/types/company";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  company: Company;
};

export function CompanyIdentityCard({ company }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("companies.detail.sections.identity")}>
      <DefinitionList
        items={[
          {
            label: t("companies.fields.legalName"),
            value: company.legalName,
          },
          {
            label: t("companies.fields.tradeName"),
            value: company.tradeName ?? EMPTY,
          },
          {
            label: t("companies.fields.taxId"),
            value: company.taxId ?? EMPTY,
          },
        ]}
      />
    </Card>
  );
}
