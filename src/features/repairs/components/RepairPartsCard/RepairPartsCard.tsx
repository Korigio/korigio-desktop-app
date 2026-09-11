import type { Repair } from "@/features/repairs/types/repair";
import { partsCardStatus } from "@/features/repairs/utils/repairWorkflow";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Card } from "@/ui";

type Props = {
  repair: Repair;
  onConfirmClick: () => void;
};

export function RepairPartsCard({ repair, onConfirmClick }: Props) {
  const { t } = useI18n();
  const cardStatus = partsCardStatus(repair);

  if (cardStatus === "disabled") {
    return null;
  }

  return (
    <Card title={t("repairs.workflow.cards.parts.title")}>
      {cardStatus === "active" ? (
        <div className="flex flex-col gap-3">
          <p className="text-sm text-muted">
            {t("repairs.workflow.cards.parts.confirmHint")}
          </p>
          <div>
            <Button type="button" onClick={onConfirmClick}>
              {t("repairs.workflow.cards.parts.confirmButton")}
            </Button>
          </div>
        </div>
      ) : (
        <p className="text-sm text-muted">
          {t("repairs.workflow.cards.parts.allConfirmed")}
        </p>
      )}
    </Card>
  );
}
