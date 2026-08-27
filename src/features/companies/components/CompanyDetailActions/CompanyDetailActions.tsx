import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import { Button, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  company: Company;
  onCompanyChange: (company: Company) => void;
  onBack: () => void;
};

export function CompanyDetailActions({
  company,
  onCompanyChange,
  onBack,
}: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(company.archivedAt);

  return (
    <>
      <LinkButton to={`/companies/${company.id}/edit`} variant="secondary">
        {t("companies.actions.edit")}
      </LinkButton>
      {!isArchived && !company.isDefault ? (
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void (async () => {
              const next = await companiesApi.setDefault(company.id);
              onCompanyChange(next);
            })();
          }}
        >
          {t("companies.actions.setDefault")}
        </Button>
      ) : null}
      <Button
        type="button"
        variant="secondary"
        onClick={() => {
          void (async () => {
            const next = isArchived
              ? await companiesApi.unarchive(company.id)
              : await companiesApi.archive(company.id);
            onCompanyChange(next);
          })();
        }}
      >
        {isArchived
          ? t("companies.actions.unarchive")
          : t("companies.actions.archive")}
      </Button>
      <Button type="button" variant="secondary" onClick={onBack}>
        {t("companies.backToList")}
      </Button>
    </>
  );
}
