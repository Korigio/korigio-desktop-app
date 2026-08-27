import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { Button, LinkButton } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repair: Repair;
  onRepairChange: (repair: Repair) => void;
  onBack: () => void;
};

export function RepairDetailActions({
  repair,
  onRepairChange,
  onBack,
}: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(repair.archivedAt);
  const isCancelled = repair.status === "cancelled";

  return (
    <>
      {!isArchived ? (
        <LinkButton to={`/repairs/${repair.id}/edit`} variant="secondary">
          {t("repairs.actions.edit")}
        </LinkButton>
      ) : null}
      <LinkButton to={`/repairs/${repair.id}/print`} variant="secondary">
        {t("repairs.actions.print")}
      </LinkButton>
      {!isCancelled && !isArchived ? (
        <Button
          type="button"
          variant="secondary"
          onClick={() => {
            void (async () => {
              const next = await repairsApi.update(repair.id, {
                customerId: repair.customerId,
                deviceId: repair.deviceId,
                status: "cancelled",
              });
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
