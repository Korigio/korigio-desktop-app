import type { Company } from "@/features/companies/types/company";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  company: Company;
};

export function CompanyDetailStatusPanel({ company }: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(company.archivedAt);

  const statusLabel = isArchived
    ? t("companies.status.archived")
    : company.isDefault
      ? t("companies.status.default")
      : t("companies.status.active");

  return (
    <Card className="border-primary/20 bg-surface">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-xs font-medium uppercase tracking-wide text-muted">
            {t("companies.fields.status")}
          </p>
          <p
            className={cn(
              "text-2xl font-semibold sm:text-3xl",
              isArchived ? "text-muted" : "text-foreground",
            )}
          >
            {statusLabel}
          </p>
        </div>
        <div className="flex flex-col gap-1 text-sm text-muted sm:text-right">
          <p>
            {t("companies.fields.createdAt")}: {company.createdAt.slice(0, 10)}
          </p>
          <p>
            {t("companies.fields.updatedAt")}: {company.updatedAt.slice(0, 10)}
          </p>
        </div>
      </div>
    </Card>
  );
}
