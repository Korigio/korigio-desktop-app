import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { collectedAtOrToday } from "@/features/repairs/utils/isoDate";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, StatusMessage } from "@/ui";

type Props = {
  repair: Repair;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
};

export function RecordPickupModal({
  repair,
  open: isOpen,
  onOpenChange,
  onSuccess,
}: Props) {
  const { t } = useI18n();
  const { busy, error, run } = useModalAsyncAction(isOpen);

  async function handleConfirm() {
    const collectedAt = collectedAtOrToday(repair.collectedAt);
    const updated = await run(
      () => repairsApi.completePickup(repair.id, collectedAt),
      t("repairs.workflow.modals.recordPickup.recordFailed"),
    );
    if (updated) {
      onSuccess(updated);
      onOpenChange(false);
    }
  }

  return (
    <Dialog
      open={isOpen}
      onOpenChange={onOpenChange}
      title={t("repairs.workflow.modals.recordPickup.title")}
      description={t("repairs.workflow.modals.recordPickup.description")}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-4">
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            disabled={busy}
            onClick={() => void handleConfirm()}
          >
            {busy
              ? t("common.saving")
              : t("repairs.workflow.modals.recordPickup.confirmButton")}
          </Button>
          <Button
            type="button"
            variant="secondary"
            disabled={busy}
            onClick={() => onOpenChange(false)}
          >
            {t("repairs.workflow.modals.cancel")}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
