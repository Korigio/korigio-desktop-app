import type { Repair } from "@/features/repairs/types/repair";
import {
  MODAL_WORKFLOW_ACTIONS,
  repairWorkflowActionKey,
  workflowActionHref,
  type RepairWorkflowActionKey,
} from "@/features/repairs/utils/repairWorkflow";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Card, LinkButton } from "@/ui";

type Props = {
  repair: Repair;
  onModalAction: (key: RepairWorkflowActionKey) => void;
};

export function RepairWorkflowBanner({ repair, onModalAction }: Props) {
  const { t } = useI18n();
  const actionKey = repairWorkflowActionKey(repair);

  if (!actionKey) {
    return null;
  }

  const label = t(`repairs.workflow.actions.${actionKey}`);

  return (
    <Card className="border-primary/30 bg-primary/5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="text-xs font-medium uppercase tracking-wide text-primary">
            {t("repairs.workflow.banner.nextStep")}
          </p>
          <p className="text-base font-semibold text-foreground">{label}</p>
        </div>
        {MODAL_WORKFLOW_ACTIONS.has(actionKey) ? (
          <Button type="button" onClick={() => onModalAction(actionKey)}>
            {label}
          </Button>
        ) : (
          <LinkButton to={workflowActionHref(repair.id, actionKey)}>
            {label}
          </LinkButton>
        )}
      </div>
    </Card>
  );
}
