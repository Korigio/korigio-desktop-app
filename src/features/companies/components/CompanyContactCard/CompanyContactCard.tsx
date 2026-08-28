import type { Company } from "@/features/companies/types/company";
import { Card, DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  company: Company;
};

export function CompanyContactCard({ company }: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("companies.detail.sections.contact")}>
      <DefinitionList
        items={[
          {
            label: t("companies.fields.address"),
            value: company.address ?? EMPTY,
          },
          {
            label: t("companies.fields.phone"),
            value: company.phone ?? EMPTY,
          },
          {
            label: t("companies.fields.email"),
            value: company.email ?? EMPTY,
          },
          {
            label: t("companies.fields.website"),
            value: company.website ?? EMPTY,
          },
        ]}
      />
    </Card>
  );
}
