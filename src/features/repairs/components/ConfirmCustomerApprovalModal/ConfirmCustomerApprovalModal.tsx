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

export function ConfirmCustomerApprovalModal(props: Props) {
  const { t } = useI18n();
  const messages: ConfirmSignedDocumentMessages = {
    title: t("repairs.workflow.modals.confirmCustomerApproval.title"),
    description: t("repairs.workflow.modals.confirmCustomerApproval.description"),
    hint: t("repairs.workflow.modals.confirmCustomerApproval.hint"),
    printButton: t(
      "repairs.workflow.modals.confirmCustomerApproval.printDiagnosis",
    ),
    confirmButton: t(
      "repairs.workflow.modals.confirmCustomerApproval.confirmButton",
    ),
    continueWithoutUpload: t(
      "repairs.workflow.modals.confirmCustomerApproval.continueWithoutUpload",
    ),
    confirming: t(
      "repairs.workflow.modals.confirmCustomerApproval.confirming",
    ),
    confirmFailed: t(
      "repairs.workflow.modals.confirmCustomerApproval.confirmFailed",
    ),
  };

  return (
    <ConfirmSignedDocumentModal
      {...props}
      documentType="diagnosisSigned"
      printTo={`/repairs/${props.repair.id}/print/diagnosis`}
      messages={messages}
      onConfirm={(repairId) => repairsApi.confirmCustomerApproval(repairId)}
    />
  );
}
