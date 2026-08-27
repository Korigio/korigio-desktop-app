import { invoke } from "@/shared/api/invoke";
import type { RepairPrintReport } from "@/features/print/types/printReport";

export const printApi = {
  getRepairReport(repairId: number): Promise<RepairPrintReport> {
    return invoke<RepairPrintReport>("get_repair_print_report", { repairId });
  },
};
