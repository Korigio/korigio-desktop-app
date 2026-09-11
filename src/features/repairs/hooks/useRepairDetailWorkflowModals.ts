import { useCallback, useEffect, useState } from "react";
import type { SetURLSearchParams } from "react-router";
import type { Repair } from "@/features/repairs/types/repair";
import type { RepairDocumentType } from "@/features/repairs/types/repairDocument";
import type { RepairWorkflowActionKey } from "@/features/repairs/utils/repairWorkflow";

export type RepairDetailModalKey =
  | "confirmIntake"
  | "diagnose"
  | "confirmCustomerApproval"
  | "confirmParts"
  | "writeProtocol"
  | "confirmSummary"
  | "recordPickup";
export type RepairUploadModalState = {
  documentType: RepairDocumentType;
} | null;

const ACTION_MODAL: Record<string, RepairDetailModalKey> = {
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

export function modalKeyFromAction(
  action: string | null,
): RepairDetailModalKey | null {
  return action ? (ACTION_MODAL[action] ?? null) : null;
}

export function useRepairDetailWorkflowModals(
  searchParams: URLSearchParams,
  setSearchParams: SetURLSearchParams,
  setRepair: (repair: Repair) => void,
) {
  const [openModal, setOpenModal] = useState<RepairDetailModalKey | null>(null);
  const [uploadModal, setUploadModal] = useState<RepairUploadModalState>(null);

  useEffect(() => {
    const modal = modalKeyFromAction(searchParams.get("action"));
    if (!modal) return;
    setOpenModal(modal);
    const next = new URLSearchParams(searchParams);
    next.delete("action");
    setSearchParams(next, { replace: true });
  }, [searchParams, setSearchParams]);

  const openWorkflowAction = useCallback((key: RepairWorkflowActionKey) => {
    const modal = modalKeyFromAction(key);
    if (modal) setOpenModal(modal);
  }, []);

  const repairUpdated = useCallback(
    (next: Repair) => setRepair(next),
    [setRepair],
  );
  const diagnosisFinalized = useCallback(
    (next: Repair) => {
      setRepair(next);
      if (next.status === "waiting_customer")
        setOpenModal("confirmCustomerApproval");
    },
    [setRepair],
  );
  const protocolCompleted = useCallback(
    (next: Repair) => {
      setRepair(next);
      if (next.status === "ready") setOpenModal("confirmSummary");
    },
    [setRepair],
  );
  const summaryConfirmed = useCallback(
    (next: Repair) => {
      setRepair(next);
      if (next.status === "awaiting_pickup") setOpenModal("recordPickup");
    },
    [setRepair],
  );

  return {
    openModal,
    setOpenModal,
    uploadModal,
    setUploadModal,
    openWorkflowAction,
    repairUpdated,
    diagnosisFinalized,
    protocolCompleted,
    summaryConfirmed,
  };
}
