import type { Repair } from "@/features/repairs/types/repair";
import { protocolCardStatus } from "@/features/repairs/utils/repairWorkflow";
import { WorkflowCardStatusBadge } from "@/features/repairs/components/WorkflowCardStatusBadge";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Card, DefinitionList } from "@/ui";

type Props = {
  repair: Repair;
  onWriteClick: () => void;
};

export function RepairProtocolCard({ repair, onWriteClick }: Props) {
  const { t } = useI18n();
  const cardStatus = protocolCardStatus(repair);
  const empty = t("print.emptyValue");

  if (cardStatus === "disabled") {
    return null;
  }

  const hasWork = Boolean(repair.workPerformed?.trim());

  return (
    <Card
      title={t("repairs.workflow.cards.protocol.title")}
      actions={<WorkflowCardStatusBadge status={cardStatus} />}
    >
      {hasWork ? (
        <DefinitionList
          items={[
            {
              label: t("repairs.workflow.cards.protocol.workPerformed"),
              value: repair.workPerformed?.trim() || empty,
            },
          ]}
        />
      ) : cardStatus === "active" ? (
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">
            {t("repairs.workflow.cards.protocol.recordHint")}
          </p>
          <div>
            <Button type="button" onClick={onWriteClick}>
              {t("repairs.workflow.cards.protocol.writeButton")}
            </Button>
          </div>
        </div>
      ) : (
        <p className="text-sm text-muted">
          {t("repairs.workflow.cards.protocol.noneYet")}
        </p>
      )}
    </Card>
  );
}
