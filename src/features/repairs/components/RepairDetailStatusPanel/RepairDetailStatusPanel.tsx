import { RepairProgressBar } from "@/features/repairs/components/RepairProgressBar";
import type { Repair } from "@/features/repairs/types/repair";
import { isRepairCompleted } from "@/features/repairs/utils/repairDiagnosis";
import { Card, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  repair: Repair;
};

export function RepairDetailStatusPanel({ repair }: Props) {
  const { t } = useI18n();
  const isCancelled = repair.status === "cancelled";
  const isArchived = Boolean(repair.archivedAt);
  const isCompleted = isRepairCompleted(repair);

  return (
    <Card className="border-primary/20 bg-surface">
      <div className="flex flex-col gap-4">
        {isCompleted ? (
          <StatusMessage tone="success">
            {t("repairs.detail.completedMessage")}
          </StatusMessage>
        ) : null}
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-xs font-medium uppercase tracking-wide text-muted">
            {t("repairs.fields.status")}
          </p>
          <p
            className={cn(
              "text-2xl font-semibold sm:text-3xl",
              isCancelled ? "text-muted line-through" : "text-foreground",
            )}
          >
            {t(`repairs.status.${repair.status}`)}
          </p>
          {isArchived ? (
            <p className="text-sm text-muted">{t("repairs.detail.archived")}</p>
          ) : null}
        </div>
        <RepairProgressBar status={repair.status} />
      </div>
    </Card>
  );
}
