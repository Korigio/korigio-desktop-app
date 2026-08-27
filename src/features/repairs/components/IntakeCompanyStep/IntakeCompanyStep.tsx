import {
  companyLabel,
  type Company,
} from "@/features/companies/types/company";
import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeCompanyStep({ intake }: Props) {
  const { t } = useI18n();
  const selectedId = intake.company?.id;

  return (
    <div className="flex flex-col gap-3">
      <h2 className="text-base font-medium">
        {t("repairs.intake.company.heading")}
      </h2>
      <p className="text-sm text-muted">{t("repairs.intake.company.hint")}</p>
      <ul className="flex flex-col gap-2" role="radiogroup" aria-label={t("repairs.fields.company")}>
        {intake.companies.map((company) => (
          <CompanyOption
            key={company.id}
            company={company}
            selected={company.id === selectedId}
            onSelect={() => intake.selectCompany(company)}
            defaultLabel={t("companies.status.default")}
          />
        ))}
      </ul>
    </div>
  );
}

function CompanyOption({
  company,
  selected,
  onSelect,
  defaultLabel,
}: {
  company: Company;
  selected: boolean;
  onSelect: () => void;
  defaultLabel: string;
}) {
  const inputId = `intake-company-${company.id}`;
  return (
    <li>
      <label
        htmlFor={inputId}
        className="flex cursor-pointer items-start gap-3 rounded-md border border-border px-3 py-2 hover:bg-background"
      >
        <input
          id={inputId}
          type="radio"
          name="intake-company"
          className="mt-1"
          checked={selected}
          onChange={onSelect}
        />
        <span className="min-w-0 flex-1">
          <span className="block font-medium">{companyLabel(company)}</span>
          {company.legalName !== companyLabel(company) ? (
            <span className="block text-sm text-muted">{company.legalName}</span>
          ) : null}
          {company.isDefault ? (
            <span className="mt-1 inline-block text-xs text-muted">
              {defaultLabel}
            </span>
          ) : null}
        </span>
      </label>
    </li>
  );
}
