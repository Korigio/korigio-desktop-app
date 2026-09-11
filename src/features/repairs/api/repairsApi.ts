import { invoke } from "@/shared/api/invoke";
import type {
  CompleteRepairDiagnosisInput,
  CompleteRepairDiagnosisResult,
  Repair,
  RepairInput,
  RepairListQuery,
  RepairListResult,
} from "@/features/repairs/types/repair";
import type {
  RepairDocument,
  RepairDocumentType,
} from "@/features/repairs/types/repairDocument";

export const repairsApi = {
  list(query: RepairListQuery = {}): Promise<RepairListResult> {
    return invoke<RepairListResult>("list_repairs", { query });
  },
  get(id: string): Promise<Repair> {
    return invoke<Repair>("get_repair", { id });
  },
  create(input: RepairInput): Promise<Repair> {
    return invoke<Repair>("create_repair", { input });
  },
  update(id: string, input: RepairInput): Promise<Repair> {
    return invoke<Repair>("update_repair", { id, input });
  },
  completeDiagnosis(
    input: CompleteRepairDiagnosisInput,
  ): Promise<CompleteRepairDiagnosisResult> {
    return invoke<CompleteRepairDiagnosisResult>("complete_repair_diagnosis", {
      input,
    });
  },
  listDocuments(repairId: string): Promise<RepairDocument[]> {
    return invoke<RepairDocument[]>("list_repair_documents", { repairId });
  },
  uploadDocument(
    repairId: string,
    documentType: RepairDocumentType,
    sourcePath: string,
  ): Promise<RepairDocument> {
    return invoke<RepairDocument>("upload_repair_document", {
      repairId,
      documentType,
      sourcePath,
    });
  },
  deleteDocument(
    repairId: string,
    documentType: RepairDocumentType,
  ): Promise<void> {
    return invoke<void>("delete_repair_document", { repairId, documentType });
  },
  openDocument(
    repairId: string,
    documentType: RepairDocumentType,
  ): Promise<void> {
    return invoke<void>("open_repair_document", { repairId, documentType });
  },
  confirmCustomerApproval(repairId: string): Promise<Repair> {
    return invoke<Repair>("confirm_customer_approval", { repairId });
  },
  confirmIntake(repairId: string): Promise<Repair> {
    return invoke<Repair>("confirm_repair_intake", { repairId });
  },
  confirmSummary(repairId: string): Promise<Repair> {
    return invoke<Repair>("confirm_repair_summary", { repairId });
  },
  recordSummaryHandover(
    repairId: string,
    collectedAt: string,
    warrantyYears: number,
  ): Promise<Repair> {
    return invoke<Repair>("record_repair_summary_handover", {
      repairId,
      collectedAt,
      warrantyYears,
    });
  },
  confirmPartsReceived(repairId: string): Promise<Repair> {
    return invoke<Repair>("confirm_repair_parts_received", { repairId });
  },
  completeProtocol(repairId: string, workPerformed: string): Promise<Repair> {
    return invoke<Repair>("complete_repair_protocol", {
      repairId,
      workPerformed,
    });
  },
  completePickup(repairId: string, collectedAt: string): Promise<Repair> {
    return invoke<Repair>("complete_repair_pickup", { repairId, collectedAt });
  },
  assign(repairId: string, staffId: string): Promise<Repair> {
    return invoke<Repair>("assign_repair", { repairId, staffId });
  },
  takeOver(repairId: string): Promise<Repair> {
    return invoke<Repair>("take_over_repair", { repairId });
  },
};
