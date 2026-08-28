import { useCompanyLogo } from "@/features/companies/hooks/useCompanyLogo";
import type { Company } from "@/features/companies/types/company";
import { Button, Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  company: Company;
  onCompanyChange: (company: Company) => void;
  disabled?: boolean;
};

export function CompanyLogoCard({
  company,
  onCompanyChange,
  disabled = false,
}: Props) {
  const { t } = useI18n();
  const logo = useCompanyLogo({ company, onCompanyChange, disabled });

  return (
    <Card title={t("companies.detail.sections.logo")}>
      {logo.logoUrl ? (
        <img
          src={logo.logoUrl}
          alt={t("companies.logo.alt")}
          className="h-20 w-auto max-w-xs object-contain"
        />
      ) : (
        <p className="text-sm text-muted">{t("companies.logo.empty")}</p>
      )}
      {logo.error ? (
        <StatusMessage tone="danger">{logo.error}</StatusMessage>
      ) : null}
      <div className="mt-3 flex flex-wrap gap-2">
        <Button
          type="button"
          variant="secondary"
          disabled={disabled || logo.busy}
          onClick={() => void logo.uploadLogo()}
        >
          {logo.hasLogo
            ? t("companies.logo.replace")
            : t("companies.logo.upload")}
        </Button>
        {logo.hasLogo ? (
          <Button
            type="button"
            variant="secondary"
            disabled={disabled || logo.busy}
            onClick={() => void logo.clearLogo()}
          >
            {t("companies.logo.clear")}
          </Button>
        ) : null}
      </div>
    </Card>
  );
}
