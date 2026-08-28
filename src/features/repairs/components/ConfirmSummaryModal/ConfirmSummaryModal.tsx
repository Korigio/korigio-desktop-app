import { repairsApi } from "@/features/repairs/api/repairsApi";
import {
  ConfirmSignedDocumentModal,
  type ConfirmSignedDocumentMessages,
} from "@/features/repairs/components/ConfirmSignedDocumentModal";
import type { Repair } from "@/features/repairs/types/repair";
import type { RepairDocument } from "@/features/repairs/types/repairDocument";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repair: Repair;
  documents: RepairDocument[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (repair: Repair) => void;
  onDocumentsChange: () => void;
};

export function ConfirmSummaryModal(props: Props) {
  const { t } = useI18n();
  const messages: ConfirmSignedDocumentMessages = {
    title: t("repairs.workflow.modals.confirmSummary.title"),
    description: t("repairs.workflow.modals.confirmSummary.description"),
    hint: t("repairs.workflow.modals.confirmSummary.hint"),
    printButton: t("repairs.workflow.modals.confirmSummary.printSummary"),
    confirmButton: t("repairs.workflow.modals.confirmSummary.confirmButton"),
    continueWithoutUpload: t(
      "repairs.workflow.modals.confirmSummary.continueWithoutUpload",
    ),
    confirming: t("repairs.workflow.modals.confirmSummary.confirming"),
    confirmFailed: t("repairs.workflow.modals.confirmSummary.confirmFailed"),
  };

  return (
    <ConfirmSignedDocumentModal
      {...props}
      documentType="summarySigned"
      printTo={`/repairs/${props.repair.id}/print/summary`}
      messages={messages}
      onConfirm={(repairId) => repairsApi.confirmSummary(repairId)}
    />
  );
}
