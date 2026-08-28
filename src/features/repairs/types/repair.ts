export const REPAIR_STATUSES = [
  "received",
  "diagnosis",
  "waiting_customer",
  "waiting_part",
  "in_repair",
  "ready",
  "awaiting_pickup",
  "collected",
  "cancelled",
] as const;

export type RepairStatus = (typeof REPAIR_STATUSES)[number];

export type Repair = {
  id: number;
  repairNumber: string;
  customerId: number;
  deviceId: number;
  /** Present after create; may be null on legacy rows. */
  companyId: number | null;
  status: RepairStatus;
  receivedAt: string;
  reportedProblem: string | null;
  accessoriesReceived: string | null;
  deviceCondition: string | null;
  expectedPickupAt: string | null;
  diagnosisNotes: string | null;
  workPerformed: string | null;
  notes: string | null;
  /** Pre-tax estimate in integer cents; null until first set via diagnosis flow. */
  estimateBaseCents: number | null;
  /** Tax rate snapshotted at estimate time, in basis points (e.g. 19% → 1900). */
  estimateTaxRateBps: number | null;
  estimateTaxCents: number | null;
  estimateGrossCents: number | null;
  readyAt: string | null;
  collectedAt: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type CompleteDiagnosisMode = "draft" | "finalize";

export type CompleteRepairDiagnosisInput = {
  repairId: number;
  mode: CompleteDiagnosisMode;
  diagnosisNotes: string | null;
  expectedPickupAt: string | null;
  /** Null only when the repair has never had an estimate; clearing is rejected. */
  estimateBaseCents: number | null;
};

export type CompleteRepairDiagnosisResult = {
  repair: Repair;
};

export type RepairInput = {
  customerId: number;
  deviceId: number;
  /** Required on create; ignored on update. */
  companyId: number;
  status?: RepairStatus | null;
  reportedProblem?: string | null;
  accessoriesReceived?: string | null;
  deviceCondition?: string | null;
  expectedPickupAt?: string | null;
  diagnosisNotes?: string | null;
  workPerformed?: string | null;
  notes?: string | null;
};

export type RepairListQuery = {
  query?: string;
  customerId?: number;
  deviceId?: number;
  status?: RepairStatus;
  page?: number;
  pageSize?: number;
};

export type RepairListItem = Repair & {
  customerName: string;
};

export type RepairListResult = {
  items: RepairListItem[];
  total: number;
  page: number;
  pageSize: number;
};

export function repairToInput(
  repair: Repair,
  overrides: Partial<RepairInput> = {},
): RepairInput {
  return {
    customerId: repair.customerId,
    deviceId: repair.deviceId,
    companyId: repair.companyId ?? 0,
    status: repair.status,
    reportedProblem: repair.reportedProblem,
    accessoriesReceived: repair.accessoriesReceived,
    deviceCondition: repair.deviceCondition,
    expectedPickupAt: repair.expectedPickupAt,
    diagnosisNotes: repair.diagnosisNotes,
    workPerformed: repair.workPerformed,
    notes: repair.notes,
    ...overrides,
  };
}
