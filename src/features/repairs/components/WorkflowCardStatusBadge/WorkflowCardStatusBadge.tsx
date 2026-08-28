import type { WorkflowCardStatus } from "@/features/repairs/utils/repairWorkflow";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

const STATUS_CLASSES: Record<WorkflowCardStatus, string> = {
  pending: "bg-background text-muted",
  active: "bg-primary/10 text-primary",
  done: "bg-emerald-500/10 text-emerald-700",
  disabled: "bg-background text-muted/70",
};

type Props = {
  status: WorkflowCardStatus;
};

export function WorkflowCardStatusBadge({ status }: Props) {
  const { t } = useI18n();

  return (
    <span
      className={cn(
        "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
        STATUS_CLASSES[status],
      )}
    >
      {t(`repairs.workflow.cardStatus.${status}`)}
    </span>
  );
}
