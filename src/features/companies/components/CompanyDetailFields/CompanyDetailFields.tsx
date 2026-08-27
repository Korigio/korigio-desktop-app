import type { Company } from "@/features/companies/types/company";
import { DefinitionList } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  company: Company;
};

export function CompanyDetailFields({ company }: Props) {
  const { t } = useI18n();

  return (
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
        {
          label: t("companies.fields.default"),
          value: company.isDefault
            ? t("companies.status.default")
            : t("companies.status.notDefault"),
        },
      ]}
    />
  );
}
