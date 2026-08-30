import { RepairDiagnosisFlowForm } from "@/features/repairs/components/RepairDiagnosisFlowForm";
import { useRepairDiagnosisFlow } from "@/features/repairs/hooks/useRepairDiagnosisFlow";
import type { Repair } from "@/features/repairs/types/repair";
import {
  isDiagnosisComplete,
  isIntakePending,
} from "@/features/repairs/utils/repairDiagnosis";
import { useI18n } from "@/shared/hooks/useI18n";
import { Dialog, StatusMessage } from "@/ui";

type Props = {
  repairId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
  onFinalized?: (repair: Repair) => void;
};

export function RepairDiagnosisModal({
  repairId,
  open,
  onOpenChange,
  onSuccess,
  onFinalized,
}: Props) {
  const { t } = useI18n();
  const flow = useRepairDiagnosisFlow(repairId, {
    enabled: open,
    onFinalizeSuccess: (repair) => {
      onSuccess(repair);
      onFinalized?.(repair);
      onOpenChange(false);
    },
    onDraftSuccess: onSuccess,
  });

  const title =
    flow.repair && isDiagnosisComplete(flow.repair)
      ? t("repairs.diagnosisFlow.titleAdjust")
      : t("repairs.diagnosisFlow.title");

  const description = flow.repair
    ? t("repairs.diagnosisFlow.subtitle").replace(
        "{number}",
        flow.repair.repairNumber,
      )
    : undefined;

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={title}
      description={description}
      closeLabel={t("common.close")}
      className="w-[min(100%-2rem,42rem)]"
    >
      {flow.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : flow.repair && isIntakePending(flow.repair) ? (
        <StatusMessage>{t("repairs.detail.intakePendingHint")}</StatusMessage>
      ) : (
        <div className="flex flex-col gap-4">
          {flow.error ? (
            <StatusMessage tone="danger">{flow.error}</StatusMessage>
          ) : null}
          {flow.success ? (
            <StatusMessage tone="success">{flow.success}</StatusMessage>
          ) : null}
          <RepairDiagnosisFlowForm flow={flow} variant="modal" />
        </div>
      )}
    </Dialog>
  );
}
