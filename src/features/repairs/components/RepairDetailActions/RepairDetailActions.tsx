import { repairsApi } from "@/features/repairs/api/repairsApi";
import { repairToInput, type Repair } from "@/features/repairs/types/repair";
import { Button } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repair: Repair;
  onRepairChange: (repair: Repair) => void;
  onBack: () => void;
};

export function RepairDetailActions({ repair, onRepairChange, onBack }: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(repair.archivedAt);
  const isCancelled = repair.status === "cancelled";

  return (
    <>
      {!isCancelled && !isArchived ? (
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void (async () => {
              const next = await repairsApi.update(
                repair.id,
                repairToInput(repair, { status: "cancelled" }),
              );
              onRepairChange(next);
            })();
          }}
        >
          {t("repairs.actions.cancel")}
        </Button>
      ) : null}
      <Button type="button" variant="secondary" onClick={onBack}>
        {t("repairs.backToList")}
      </Button>
    </>
  );
}
