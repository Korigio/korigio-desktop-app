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

export function ConfirmIntakeModal(props: Props) {
  const { t } = useI18n();
  const messages: ConfirmSignedDocumentMessages = {
    title: t("repairs.workflow.modals.confirmIntake.title"),
    description: t("repairs.workflow.modals.confirmIntake.description"),
    hint: t("repairs.workflow.modals.confirmIntake.hint"),
    printButton: t("repairs.workflow.modals.confirmIntake.printEntrance"),
    confirmButton: t("repairs.workflow.modals.confirmIntake.confirmButton"),
    continueWithoutUpload: t(
      "repairs.workflow.modals.confirmIntake.continueWithoutUpload",
    ),
    confirming: t("repairs.workflow.modals.confirmIntake.confirming"),
    confirmFailed: t("repairs.workflow.modals.confirmIntake.confirmFailed"),
  };

  return (
    <ConfirmSignedDocumentModal
      {...props}
      documentType="entranceSigned"
      printTo={`/repairs/${props.repair.id}/print`}
      messages={messages}
      onConfirm={(repairId) => repairsApi.confirmIntake(repairId)}
    />
  );
}
