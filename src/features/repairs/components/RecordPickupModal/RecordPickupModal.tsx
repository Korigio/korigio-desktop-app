import { useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { useModalAsyncAction } from "@/shared/hooks/useModalAsyncAction";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, FormField, StatusMessage, TextField } from "@/ui";

type Props = {
  repair: Repair;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
};

function todayIsoDate(): string {
  return new Date().toISOString().slice(0, 10);
}

export function RecordPickupModal({
  repair,
  open: isOpen,
  onOpenChange,
  onSuccess,
}: Props) {
  const { t } = useI18n();
  const [collectedAt, setCollectedAt] = useState(todayIsoDate);
  const { busy, error, setError, run } = useModalAsyncAction(isOpen);

  async function handleConfirm() {
    if (!collectedAt.trim()) {
      setError(t("repairs.workflow.modals.recordPickup.dateRequired"));
      return;
    }
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
        <FormField
          label={t("repairs.workflow.modals.recordPickup.pickupDate")}
          htmlFor="pickup-date"
        >
          <TextField
            id="pickup-date"
            name="collectedAt"
            type="date"
            value={collectedAt}
            disabled={busy}
            onChange={(event) => setCollectedAt(event.target.value)}
          />
        </FormField>
        <div className="flex flex-wrap gap-2">
          <Button type="button" disabled={busy} onClick={() => void handleConfirm()}>
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
