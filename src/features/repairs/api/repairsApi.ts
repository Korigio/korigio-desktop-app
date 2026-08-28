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
  get(id: number): Promise<Repair> {
    return invoke<Repair>("get_repair", { id });
  },
  create(input: RepairInput): Promise<Repair> {
    return invoke<Repair>("create_repair", { input });
  },
  update(id: number, input: RepairInput): Promise<Repair> {
    return invoke<Repair>("update_repair", { id, input });
  },
  completeDiagnosis(
    input: CompleteRepairDiagnosisInput,
  ): Promise<CompleteRepairDiagnosisResult> {
    return invoke<CompleteRepairDiagnosisResult>("complete_repair_diagnosis", {
      input,
    });
  },
  listDocuments(repairId: number): Promise<RepairDocument[]> {
    return invoke<RepairDocument[]>("list_repair_documents", { repairId });
  },
  uploadDocument(
    repairId: number,
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
    repairId: number,
    documentType: RepairDocumentType,
  ): Promise<void> {
    return invoke<void>("delete_repair_document", { repairId, documentType });
  },
  openDocument(
    repairId: number,
    documentType: RepairDocumentType,
  ): Promise<void> {
    return invoke<void>("open_repair_document", { repairId, documentType });
  },
  confirmCustomerApproval(repairId: number): Promise<Repair> {
    return invoke<Repair>("confirm_customer_approval", { repairId });
  },
  confirmIntake(repairId: number): Promise<Repair> {
    return invoke<Repair>("confirm_repair_intake", { repairId });
  },
  confirmSummary(repairId: number): Promise<Repair> {
    return invoke<Repair>("confirm_repair_summary", { repairId });
  },
  confirmPartsReceived(repairId: number): Promise<Repair> {
    return invoke<Repair>("confirm_repair_parts_received", { repairId });
  },
  completeProtocol(repairId: number, workPerformed: string): Promise<Repair> {
    return invoke<Repair>("complete_repair_protocol", {
      repairId,
      workPerformed,
    });
  },
  completePickup(repairId: number, collectedAt: string): Promise<Repair> {
    return invoke<Repair>("complete_repair_pickup", { repairId, collectedAt });
  },
};
