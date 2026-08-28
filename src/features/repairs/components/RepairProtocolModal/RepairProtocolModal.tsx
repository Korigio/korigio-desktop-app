import { useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { useI18n } from "@/shared/hooks/useI18n";
import { Button, Dialog, StatusMessage, TextArea } from "@/ui";

type Props = {
  repair: Repair;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
};

export function RepairProtocolModal({
  repair,
  open: isOpen,
  onOpenChange,
  onSuccess,
}: Props) {
  const { t } = useI18n();
  const [workPerformed, setWorkPerformed] = useState(
    () => repair.workPerformed ?? "",
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSave() {
    const trimmed = workPerformed.trim();
    if (!trimmed) {
      setError(t("repairs.workflow.modals.protocol.workRequired"));
      return;
    }
    setError(null);
    setBusy(true);
    try {
      const updated = await repairsApi.completeProtocol(repair.id, trimmed);
      onSuccess(updated);
      onOpenChange(false);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("repairs.workflow.modals.protocol.saveFailed"),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog
      open={isOpen}
      onOpenChange={onOpenChange}
      title={t("repairs.workflow.modals.protocol.title")}
      description={t("repairs.workflow.modals.protocol.description")}
      closeLabel={t("common.close")}
    >
      <div className="flex flex-col gap-4">
        {error ? <StatusMessage tone="danger">{error}</StatusMessage> : null}
        <TextArea
          id="repair-protocol-work"
          name="workPerformed"
          value={workPerformed}
          rows={6}
          disabled={busy}
          onChange={(event) => setWorkPerformed(event.target.value)}
        />
        <div className="flex flex-wrap gap-2">
          <Button type="button" disabled={busy} onClick={() => void handleSave()}>
            {busy
              ? t("common.saving")
              : t("repairs.workflow.modals.protocol.saveButton")}
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
