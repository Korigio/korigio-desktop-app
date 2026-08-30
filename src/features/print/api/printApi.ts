import { invoke } from "@/shared/api/invoke";
import type {
  DiagnosisPrintReport,
  RepairPrintReport,
  SummaryPrintReport,
} from "@/features/print/types/printReport";

export const printApi = {
  getRepairReport(repairId: string): Promise<RepairPrintReport> {
    return invoke<RepairPrintReport>("get_repair_print_report", { repairId });
  },
  getDiagnosisReport(repairId: string): Promise<DiagnosisPrintReport> {
    return invoke<DiagnosisPrintReport>("get_repair_diagnosis_print_report", {
      repairId,
    });
  },
  getSummaryReport(repairId: string): Promise<SummaryPrintReport> {
    return invoke<SummaryPrintReport>("get_repair_summary_print_report", {
      repairId,
    });
  },
};
