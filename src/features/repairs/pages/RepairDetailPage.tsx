import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";
import { RepairImagesSection } from "@/features/images/components/RepairImagesSection";
import { ConfirmSummaryModal } from "@/features/repairs/components/ConfirmSummaryModal";
import { ConfirmCustomerApprovalModal } from "@/features/repairs/components/ConfirmCustomerApprovalModal";
import { ConfirmIntakeModal } from "@/features/repairs/components/ConfirmIntakeModal";
import { ConfirmPartsModal } from "@/features/repairs/components/ConfirmPartsModal";
import { RecordPickupModal } from "@/features/repairs/components/RecordPickupModal";
import { RepairClientInfoCard } from "@/features/repairs/components/RepairClientInfoCard";
import { RepairDetailActions } from "@/features/repairs/components/RepairDetailActions";
import { RepairDetailStatusPanel } from "@/features/repairs/components/RepairDetailStatusPanel";
import { RepairDiagnosisModal } from "@/features/repairs/components/RepairDiagnosisModal";
import { RepairDiagnosisSummary } from "@/features/repairs/components/RepairDiagnosisSummary";
import { RepairDocumentCard } from "@/features/repairs/components/RepairDocumentCard";
import { RepairEntranceInfoCard } from "@/features/repairs/components/RepairEntranceInfoCard";
import { RepairLoadState } from "@/features/repairs/components/RepairLoadState";
import { RepairPartsCard } from "@/features/repairs/components/RepairPartsCard";
import { RepairPickupCard } from "@/features/repairs/components/RepairPickupCard";
import { RepairProtocolCard } from "@/features/repairs/components/RepairProtocolCard";
import { RepairProtocolModal } from "@/features/repairs/components/RepairProtocolModal";
import { RepairWorkflowBanner } from "@/features/repairs/components/RepairWorkflowBanner";
import { UploadRepairDocumentModal } from "@/features/repairs/components/UploadRepairDocumentModal";
import { useRepairDetail } from "@/features/repairs/hooks/useRepairDetail";
import { useRepairDetailRelations } from "@/features/repairs/hooks/useRepairDetailRelations";
import { useRepairDocuments } from "@/features/repairs/hooks/useRepairDocuments";
import type { Repair } from "@/features/repairs/types/repair";
import type { RepairDocumentType } from "@/features/repairs/types/repairDocument";
import type { RepairWorkflowActionKey } from "@/features/repairs/utils/repairWorkflow";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { repairId: number };

type ModalKey =
  | "confirmIntake"
  | "diagnose"
  | "confirmCustomerApproval"
  | "confirmParts"
  | "writeProtocol"
  | "confirmSummary"
  | "recordPickup";

type UploadModalState = {
  documentType: RepairDocumentType;
} | null;

function modalKeyFromAction(action: string | null): ModalKey | null {
  if (
    action === "confirmIntake" ||
    action === "startDiagnose" ||
    action === "adjustDiagnosis" ||
    action === "diagnose" ||
    action === "confirmCustomerApproval" ||
    action === "confirmParts" ||
    action === "writeProtocol" ||
    action === "confirmSummary" ||
    action === "recordPickup" ||
    action === "intake" ||
    action === "approval" ||
    action === "parts" ||
    action === "protocol" ||
    action === "summary" ||
    action === "pickup"
  ) {
    const map: Record<string, ModalKey> = {
      confirmIntake: "confirmIntake",
      intake: "confirmIntake",
      startDiagnose: "diagnose",
      adjustDiagnosis: "diagnose",
      diagnose: "diagnose",
      confirmCustomerApproval: "confirmCustomerApproval",
      approval: "confirmCustomerApproval",
      confirmParts: "confirmParts",
      parts: "confirmParts",
      writeProtocol: "writeProtocol",
      protocol: "writeProtocol",
      confirmSummary: "confirmSummary",
      summary: "confirmSummary",
      recordPickup: "recordPickup",
      pickup: "recordPickup",
    };
    return map[action] ?? null;
  }
  return null;
}

export function RepairDetailPage({ repairId }: Props) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const { repair, loading, error, setRepair } = useRepairDetail(repairId);
  const relations = useRepairDetailRelations(repair);
  const {
    documents,
    loading: documentsLoading,
    reload: reloadDocuments,
  } = useRepairDocuments(repairId);
  const [openModal, setOpenModal] = useState<ModalKey | null>(null);
  const [uploadModal, setUploadModal] = useState<UploadModalState>(null);

  useEffect(() => {
    const action = searchParams.get("action");
    const modal = modalKeyFromAction(action);
    if (modal) {
      setOpenModal(modal);
      const next = new URLSearchParams(searchParams);
      next.delete("action");
      setSearchParams(next, { replace: true });
    }
  }, [searchParams, setSearchParams]);

  if (loading) {
    return <RepairLoadState />;
  }

  if (error || !repair) {
    return <RepairLoadState message={error ?? t("repairs.notFound")} />;
  }

  function handleModalAction(key: RepairWorkflowActionKey) {
    if (key === "confirmIntake") {
      setOpenModal("confirmIntake");
    } else if (key === "startDiagnose" || key === "adjustDiagnosis") {
      setOpenModal("diagnose");
    } else if (key === "confirmCustomerApproval") {
      setOpenModal("confirmCustomerApproval");
    } else if (key === "confirmParts") {
      setOpenModal("confirmParts");
    } else if (key === "writeProtocol") {
      setOpenModal("writeProtocol");
    } else if (key === "confirmSummary") {
      setOpenModal("confirmSummary");
    } else if (key === "recordPickup") {
      setOpenModal("recordPickup");
    }
  }

  function handleRepairUpdated(next: Repair) {
    setRepair(next);
  }

  function handleDocumentUploaded() {
    void reloadDocuments();
  }

  function handleDiagnosisFinalized(next: Repair) {
    setRepair(next);
    if (next.status === "waiting_customer") {
      setOpenModal("confirmCustomerApproval");
    }
  }

  function handleProtocolCompleted(next: Repair) {
    setRepair(next);
    if (next.status === "ready") {
      setOpenModal("confirmSummary");
    }
  }

  function handleSummaryConfirmed(next: Repair) {
    setRepair(next);
    if (next.status === "awaiting_pickup") {
      setOpenModal("recordPickup");
    }
  }

  return (
    <Page className="max-w-5xl">
      <PageHeader
        title={repair.repairNumber}
        description={t("repairs.detail.subtitle")}
        actions={
          <RepairDetailActions
            repair={repair}
            onRepairChange={setRepair}
            onBack={() => navigate("/repairs")}
          />
        }
      />

      <div className="flex flex-col gap-4">
        <RepairDetailStatusPanel repair={repair} />

        <RepairWorkflowBanner
          repair={repair}
          onModalAction={handleModalAction}
        />

        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2 lg:gap-6">
          <RepairClientInfoCard
            repair={repair}
            customer={relations.customer}
            device={relations.device}
            company={relations.company}
            loading={relations.loading}
          />
          <RepairEntranceInfoCard repair={repair} />
          <RepairDiagnosisSummary
            repair={repair}
            onOpenDiagnosis={() => setOpenModal("diagnose")}
          />
          <RepairDocumentCard
            repair={repair}
            documents={documents}
            documentsLoading={documentsLoading}
            onUploadClick={(documentType) => setUploadModal({ documentType })}
            onDocumentsChange={() => void reloadDocuments()}
          />
          <RepairPartsCard
            repair={repair}
            onConfirmClick={() => setOpenModal("confirmParts")}
          />
          <RepairProtocolCard
            repair={repair}
            onWriteClick={() => setOpenModal("writeProtocol")}
          />
          <RepairPickupCard
            repair={repair}
            onRecordClick={() => setOpenModal("recordPickup")}
          />
        </div>

        <RepairImagesSection repairId={repair.id} />
      </div>

      {uploadModal ? (
        <UploadRepairDocumentModal
          repairId={repair.id}
          documentType={uploadModal.documentType}
          open
          onOpenChange={(next) => {
            if (!next) {
              setUploadModal(null);
            }
          }}
          onSuccess={handleDocumentUploaded}
        />
      ) : null}

      <ConfirmIntakeModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmIntake"}
        onOpenChange={(next) => setOpenModal(next ? "confirmIntake" : null)}
        onSuccess={handleRepairUpdated}
        onDocumentsChange={() => void reloadDocuments()}
      />
      <RepairDiagnosisModal
        repairId={repair.id}
        open={openModal === "diagnose"}
        onOpenChange={(next) => setOpenModal(next ? "diagnose" : null)}
        onSuccess={handleRepairUpdated}
        onFinalized={handleDiagnosisFinalized}
      />
      <ConfirmSummaryModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmSummary"}
        onOpenChange={(next) => setOpenModal(next ? "confirmSummary" : null)}
        onSuccess={handleSummaryConfirmed}
        onDocumentsChange={() => void reloadDocuments()}
      />
      <ConfirmCustomerApprovalModal
        repair={repair}
        documents={documents}
        open={openModal === "confirmCustomerApproval"}
        onOpenChange={(next) =>
          setOpenModal(next ? "confirmCustomerApproval" : null)
        }
        onSuccess={handleRepairUpdated}
        onDocumentsChange={() => void reloadDocuments()}
      />
      <ConfirmPartsModal
        repair={repair}
        open={openModal === "confirmParts"}
        onOpenChange={(next) => setOpenModal(next ? "confirmParts" : null)}
        onSuccess={handleRepairUpdated}
      />
      <RepairProtocolModal
        repair={repair}
        open={openModal === "writeProtocol"}
        onOpenChange={(next) => setOpenModal(next ? "writeProtocol" : null)}
        onSuccess={handleProtocolCompleted}
      />
      <RecordPickupModal
        repair={repair}
        open={openModal === "recordPickup"}
        onOpenChange={(next) => setOpenModal(next ? "recordPickup" : null)}
        onSuccess={handleRepairUpdated}
      />
    </Page>
  );
}
