import type { Customer } from "@/features/customers/types/customer";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  customer: Customer;
};

export function CustomerDetailStatusPanel({ customer }: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(customer.archivedAt);

  return (
    <Card className="border-primary/20 bg-surface">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-xs font-medium uppercase tracking-wide text-muted">
            {t("customers.fields.status")}
          </p>
          <p
            className={cn(
              "text-2xl font-semibold sm:text-3xl",
              isArchived ? "text-muted" : "text-foreground",
            )}
          >
            {isArchived
              ? t("customers.status.archived")
              : t("customers.status.active")}
          </p>
        </div>
        <div className="flex flex-col gap-1 text-sm text-muted sm:text-right">
          <p>
            {t("customers.fields.createdAt")}: {customer.createdAt.slice(0, 10)}
          </p>
          <p>
            {t("customers.fields.updatedAt")}: {customer.updatedAt.slice(0, 10)}
          </p>
        </div>
      </div>
    </Card>
  );
}
