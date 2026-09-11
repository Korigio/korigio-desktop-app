import type { Repair, RepairStatus } from "@/features/repairs/types/repair";
import type {
  RepairDocument,
  RepairDocumentType,
} from "@/features/repairs/types/repairDocument";
import { REPAIR_DOCUMENT_TYPES } from "@/features/repairs/types/repairDocument";
import {
  isRepairDiagnosisBlocked,
  isDiagnosisPending,
  isIntakePending,
  isRepairCompleted,
} from "@/features/repairs/utils/repairDiagnosis";
import {
  repairProgressIndex,
  repairProgressStepIndex,
  type RepairProgressStep,
} from "@/features/repairs/utils/repairProgress";

export type RepairWorkflowActionKey =
  | "confirmIntake"
  | "startDiagnose"
  | "adjustDiagnosis"
  | "confirmCustomerApproval"
  | "confirmParts"
  | "writeProtocol"
  | "confirmSummary"
  | "recordPickup";

export type WorkflowCardStatus = "pending" | "active" | "done" | "disabled";

const WORKFLOW_ACTION_LABELS: Record<RepairWorkflowActionKey, string> = {
  confirmIntake: "Upload signed intake",
  startDiagnose: "Diagnose",
  adjustDiagnosis: "Adjust diagnosis",
  confirmCustomerApproval: "Confirm customer approval",
  confirmParts: "Confirm parts received",
  writeProtocol: "Complete repair protocol",
  confirmSummary: "Print and sign summary",
  recordPickup: "Record pickup",
};

export const MODAL_WORKFLOW_ACTIONS = new Set<RepairWorkflowActionKey>([
  "confirmIntake",
  "startDiagnose",
  "adjustDiagnosis",
  "confirmCustomerApproval",
  "confirmParts",
  "writeProtocol",
  "confirmSummary",
  "recordPickup",
]);

function atOrPast(status: RepairStatus, step: RepairProgressStep): boolean {
  return repairProgressIndex(status) >= repairProgressStepIndex(step);
}

export function repairWorkflowActionKey(
  repair: Repair,
): RepairWorkflowActionKey | null {
  if (isRepairDiagnosisBlocked(repair) || isRepairCompleted(repair)) {
    return null;
  }

  if (isIntakePending(repair)) {
    return "confirmIntake";
  }

  if (isDiagnosisPending(repair)) {
    return "startDiagnose";
  }

  switch (repair.status) {
    case "waiting_customer":
      return "confirmCustomerApproval";
    case "waiting_part":
      return "confirmParts";
    case "in_repair":
      return "writeProtocol";
    case "ready":
      return "confirmSummary";
    case "awaiting_pickup":
      return "recordPickup";
    default:
      return null;
  }
}

export function workflowActionLabel(key: RepairWorkflowActionKey): string {
  return WORKFLOW_ACTION_LABELS[key];
}

export function workflowActionHref(
  repairId: string,
  key: RepairWorkflowActionKey,
): string {
  if (MODAL_WORKFLOW_ACTIONS.has(key)) {
    return `/repairs/${repairId}?action=${key}`;
  }
  return `/repairs/${repairId}`;
}

export function documentForType(
  documents: RepairDocument[],
  documentType: RepairDocumentType,
): RepairDocument | undefined {
  return documents.find((doc) => doc.documentType === documentType);
}

export function documentCardStatus(
  repair: Repair,
  documents: RepairDocument[],
): WorkflowCardStatus {
  if (isRepairDiagnosisBlocked(repair)) {
    return "disabled";
  }
  if (documents.length === REPAIR_DOCUMENT_TYPES.length) {
    return "done";
  }
  return "active";
}

export function partsCardStatus(repair: Repair): WorkflowCardStatus {
  if (isRepairDiagnosisBlocked(repair)) {
    return "disabled";
  }
  if (atOrPast(repair.status, "in_repair")) {
    return "done";
  }
  if (repair.status === "waiting_part") {
    return "active";
  }
  if (atOrPast(repair.status, "waiting_part")) {
    return "pending";
  }
  return "disabled";
}

export function protocolCardStatus(repair: Repair): WorkflowCardStatus {
  if (isRepairDiagnosisBlocked(repair)) {
    return "disabled";
  }
  if (repair.workPerformed?.trim()) {
    return "done";
  }
  if (repair.status === "in_repair") {
    return "active";
  }
  if (atOrPast(repair.status, "signedSummary")) {
    return "done";
  }
  return "disabled";
}

export function pickupCardStatus(repair: Repair): WorkflowCardStatus {
  if (isRepairDiagnosisBlocked(repair)) {
    return "disabled";
  }
  if (repair.collectedAt || repair.status === "collected") {
    return "done";
  }
  if (repair.status === "awaiting_pickup") {
    return "active";
  }
  if (repair.status === "ready") {
    return "pending";
  }
  return "disabled";
}
