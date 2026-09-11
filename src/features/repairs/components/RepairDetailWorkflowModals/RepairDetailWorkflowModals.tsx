import { ConfirmSummaryModal } from "@/features/repairs/components/ConfirmSummaryModal";
import { ConfirmCustomerApprovalModal } from "@/features/repairs/components/ConfirmCustomerApprovalModal";
import { ConfirmIntakeModal } from "@/features/repairs/components/ConfirmIntakeModal";
import { ConfirmPartsModal } from "@/features/repairs/components/ConfirmPartsModal";
import { RecordPickupModal } from "@/features/repairs/components/RecordPickupModal";
import { RepairDiagnosisModal } from "@/features/repairs/components/RepairDiagnosisModal";
import { RepairProtocolModal } from "@/features/repairs/components/RepairProtocolModal";
import { UploadRepairDocumentModal } from "@/features/repairs/components/UploadRepairDocumentModal";
import type {
  RepairDetailModalKey,
  RepairUploadModalState,
} from "@/features/repairs/hooks/useRepairDetailWorkflowModals";
import type { Repair } from "@/features/repairs/types/repair";
import type { RepairDocument } from "@/features/repairs/types/repairDocument";

type Props = {
  repair: Repair;
  documents: RepairDocument[];
  openModal: RepairDetailModalKey | null;
  uploadModal: RepairUploadModalState;
  setOpenModal: (modal: RepairDetailModalKey | null) => void;
  closeUploadModal: () => void;
  reloadDocuments: () => void;
  onRepairUpdated: (repair: Repair) => void;
  onDiagnosisFinalized: (repair: Repair) => void;
  onProtocolCompleted: (repair: Repair) => void;
  onSummaryConfirmed: (repair: Repair) => void;
};

export function RepairDetailWorkflowModals({
  repair,
  documents,
  openModal,
  uploadModal,
  setOpenModal,
  closeUploadModal,
  reloadDocuments,
  onRepairUpdated,
  onDiagnosisFinalized,
  onProtocolCompleted,
  onSummaryConfirmed,
}: Props) {
  return (
    <>
      {uploadModal ? (
        <UploadRepairDocumentModal
          repairId={repair.id}
          documentType={uploadModal.documentType}
          open
          onOpenChange={(open) => {
            if (!open) closeUploadModal();
          }}
          onSuccess={reloadDocuments}
        />
      ) : null}
      <ConfirmIntakeModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmIntake"}
        onOpenChange={(open) => setOpenModal(open ? "confirmIntake" : null)}
        onSuccess={onRepairUpdated}
        onDocumentsChange={reloadDocuments}
      />
      <RepairDiagnosisModal
        repairId={repair.id}
        open={openModal === "diagnose"}
        onOpenChange={(open) => setOpenModal(open ? "diagnose" : null)}
        onSuccess={onRepairUpdated}
        onFinalized={onDiagnosisFinalized}
      />
      <ConfirmSummaryModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmSummary"}
        onOpenChange={(open) => setOpenModal(open ? "confirmSummary" : null)}
        onSuccess={onSummaryConfirmed}
        onDocumentsChange={reloadDocuments}
      />
      <ConfirmCustomerApprovalModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmCustomerApproval"}
        onOpenChange={(open) =>
          setOpenModal(open ? "confirmCustomerApproval" : null)
        }
        onSuccess={onRepairUpdated}
        onDocumentsChange={reloadDocuments}
      />
      <ConfirmPartsModal
        repair={repair}
        open={openModal === "confirmParts"}
        onOpenChange={(open) => setOpenModal(open ? "confirmParts" : null)}
        onSuccess={onRepairUpdated}
      />
      <RepairProtocolModal
        repair={repair}
        open={openModal === "writeProtocol"}
        onOpenChange={(open) => setOpenModal(open ? "writeProtocol" : null)}
        onSuccess={onProtocolCompleted}
      />
      <RecordPickupModal
        repair={repair}
        open={openModal === "recordPickup"}
        onOpenChange={(open) => setOpenModal(open ? "recordPickup" : null)}
        onSuccess={onRepairUpdated}
      />
    </>
  );
}
