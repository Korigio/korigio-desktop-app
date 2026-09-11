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
  id: string;
  repairNumber: string;
  customerId: string;
  deviceId: string;
  /** Present after create; may be null on legacy rows. */
  companyId: string | null;
  assignedToStaffId: string | null;
  updatedByStaffId: string | null;
  status: RepairStatus;
  receivedAt: string;
  reportedProblem: string | null;
  accessoriesReceived: string | null;
  deviceCondition: string | null;
  expectedPickupAt: string | null;
  diagnosisNotes: string | null;
  workPerformed: string | null;
  notes: string | null;
  /** Pre-discount list price from intake; null for legacy / diagnosis-set estimates. */
  estimateListCents?: number | null;
  /** Intake discount in basis points; null for legacy / diagnosis-set estimates. */
  estimateDiscountBps?: number | null;
  /** Post-discount pre-tax net in integer cents; null until set at intake or diagnosis. */
  estimateBaseCents: number | null;
  /** Tax rate snapshotted at estimate time, in basis points (e.g. 19% → 1900). */
  estimateTaxRateBps: number | null;
  estimateTaxCents: number | null;
  estimateGrossCents: number | null;
  readyAt: string | null;
  collectedAt: string | null;
  /** Warranty years set at summary handover; null until recorded. */
  warrantyYears: number | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type CompleteDiagnosisMode = "draft" | "finalize";

export type CompleteRepairDiagnosisInput = {
  repairId: string;
  mode: CompleteDiagnosisMode;
  diagnosisNotes: string | null;
  expectedPickupAt: string | null;
  /**
   * List price in integer cents (same as create_repair).
   * Null only when the repair has never had an estimate; clearing is rejected.
   */
  estimateBaseCents: number | null;
  /**
   * Discount in basis points (`0..=10000`). Omit/null = 0.
   * Invalid if set without `estimateBaseCents`.
   */
  estimateDiscountBps?: number | null;
};

export type CompleteRepairDiagnosisResult = {
  repair: Repair;
};

export type RepairInput = {
  customerId: string;
  deviceId: string;
  /** Required on create; ignored on update. */
  companyId: string;
  status?: RepairStatus | null;
  reportedProblem?: string | null;
  accessoriesReceived?: string | null;
  deviceCondition?: string | null;
  expectedPickupAt?: string | null;
  diagnosisNotes?: string | null;
  workPerformed?: string | null;
  notes?: string | null;
  /**
   * Optional intake list price in integer cents (pre-discount).
   * Omit/null = no estimate. Persisted estimate is post-discount net + tax.
   */
  estimateBaseCents?: number | null;
  /**
   * Optional discount in basis points (0..=10000). Omit/null = 0.
   * Invalid if set without `estimateBaseCents`.
   */
  estimateDiscountBps?: number | null;
};

export type RepairListQuery = {
  query?: string;
  customerId?: string;
  deviceId?: string;
  companyId?: string;
  status?: RepairStatus;
  page?: number;
  pageSize?: number;
};

export type RepairListItem = Repair & {
  customerName: string;
  deviceName: string;
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
    companyId: repair.companyId ?? "",
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
