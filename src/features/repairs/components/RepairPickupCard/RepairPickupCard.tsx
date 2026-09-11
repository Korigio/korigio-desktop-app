import type { Repair } from "@/features/repairs/types/repair";
import { pickupCardStatus } from "@/features/repairs/utils/repairWorkflow";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Card, DefinitionList } from "@/ui";

type Props = {
  repair: Repair;
  onRecordClick: () => void;
};

export function RepairPickupCard({ repair, onRecordClick }: Props) {
  const { t } = useI18n();
  const cardStatus = pickupCardStatus(repair);
  const empty = t("print.emptyValue");

  if (cardStatus === "disabled") {
    return null;
  }

  return (
    <Card title={t("repairs.workflow.cards.pickup.title")}>
      <DefinitionList
        items={[
          {
            label: t("repairs.workflow.cards.pickup.readyAt"),
            value: repair.readyAt?.slice(0, 10) ?? empty,
          },
          {
            label: t("repairs.workflow.cards.pickup.collectedAt"),
            value: repair.collectedAt?.slice(0, 10) ?? empty,
          },
        ]}
      />
      {cardStatus === "active" ? (
        <div className="mt-4">
          <Button type="button" onClick={onRecordClick}>
            {t("repairs.workflow.cards.pickup.recordButton")}
          </Button>
        </div>
      ) : null}
    </Card>
  );
}
