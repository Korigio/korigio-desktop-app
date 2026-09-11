import { useSearchParams } from "react-router";
import { RepairImagesSection } from "@/features/images/components/RepairImagesSection";
import { RepairAssigneeBar } from "@/features/repairs/components/RepairAssigneeBar";
import { RepairClientInfoCard } from "@/features/repairs/components/RepairClientInfoCard";
import { RepairDetailActions } from "@/features/repairs/components/RepairDetailActions";
import { RepairDetailStatusPanel } from "@/features/repairs/components/RepairDetailStatusPanel";
import { RepairDetailWorkflowModals } from "@/features/repairs/components/RepairDetailWorkflowModals";
import { RepairDiagnosisSummary } from "@/features/repairs/components/RepairDiagnosisSummary";
import { RepairDocumentCard } from "@/features/repairs/components/RepairDocumentCard";
import { RepairEntranceInfoCard } from "@/features/repairs/components/RepairEntranceInfoCard";
import { RepairLoadState } from "@/features/repairs/components/RepairLoadState";
import { RepairPartsCard } from "@/features/repairs/components/RepairPartsCard";
import { RepairPickupCard } from "@/features/repairs/components/RepairPickupCard";
import { RepairProtocolCard } from "@/features/repairs/components/RepairProtocolCard";
import { RepairWorkflowBanner } from "@/features/repairs/components/RepairWorkflowBanner";
import { useRepairDetail } from "@/features/repairs/hooks/useRepairDetail";
import { useRepairDetailWorkflowModals } from "@/features/repairs/hooks/useRepairDetailWorkflowModals";
import { useRepairDetailRelations } from "@/features/repairs/hooks/useRepairDetailRelations";
import { useRepairDocuments } from "@/features/repairs/hooks/useRepairDocuments";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { useNavigate } from "react-router";

type Props = { repairId: string };

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
  const modals = useRepairDetailWorkflowModals(
    searchParams,
    setSearchParams,
    setRepair,
  );

  if (loading) {
    return <RepairLoadState />;
  }

  if (error || !repair) {
    return <RepairLoadState message={error ?? t("repairs.notFound")} />;
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

        <RepairAssigneeBar repair={repair} onRepairChange={setRepair} />

        <RepairWorkflowBanner
          repair={repair}
          onModalAction={modals.openWorkflowAction}
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
            onOpenDiagnosis={() => modals.setOpenModal("diagnose")}
          />
          <RepairDocumentCard
            repair={repair}
            documents={documents}
            documentsLoading={documentsLoading}
            onUploadClick={(documentType) =>
              modals.setUploadModal({ documentType })
            }
            onDocumentsChange={() => void reloadDocuments()}
          />
          <RepairPartsCard
            repair={repair}
            onConfirmClick={() => modals.setOpenModal("confirmParts")}
          />
          <RepairProtocolCard
            repair={repair}
            onWriteClick={() => modals.setOpenModal("writeProtocol")}
          />
          <RepairPickupCard
            repair={repair}
            onRecordClick={() => modals.setOpenModal("recordPickup")}
          />
        </div>

        <RepairImagesSection repairId={repair.id} />
      </div>

      <RepairDetailWorkflowModals
        repair={repair}
        documents={documents}
        openModal={modals.openModal}
        uploadModal={modals.uploadModal}
        setOpenModal={modals.setOpenModal}
        closeUploadModal={() => modals.setUploadModal(null)}
        reloadDocuments={() => void reloadDocuments()}
        onRepairUpdated={modals.repairUpdated}
        onDiagnosisFinalized={modals.diagnosisFinalized}
        onProtocolCompleted={modals.protocolCompleted}
        onSummaryConfirmed={modals.summaryConfirmed}
      />
    </Page>
  );
}
