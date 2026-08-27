export const REPAIR_STATUSES = [
  "received",
  "diagnosis",
  "waiting_customer",
  "waiting_part",
  "in_repair",
  "ready",
  "collected",
  "cancelled",
] as const;

export type RepairStatus = (typeof REPAIR_STATUSES)[number];

export type Repair = {
  id: number;
  repairNumber: string;
  customerId: number;
  deviceId: number;
  status: RepairStatus;
  receivedAt: string;
  reportedProblem: string | null;
  accessoriesReceived: string | null;
  deviceCondition: string | null;
  diagnosisNotes: string | null;
  workPerformed: string | null;
  notes: string | null;
  readyAt: string | null;
  collectedAt: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type RepairInput = {
  customerId: number;
  deviceId: number;
  status?: RepairStatus | null;
  reportedProblem?: string | null;
  accessoriesReceived?: string | null;
  deviceCondition?: string | null;
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

export type RepairListResult = {
  items: Repair[];
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
    status: repair.status,
    reportedProblem: repair.reportedProblem,
    accessoriesReceived: repair.accessoriesReceived,
    deviceCondition: repair.deviceCondition,
    diagnosisNotes: repair.diagnosisNotes,
    workPerformed: repair.workPerformed,
    notes: repair.notes,
    ...overrides,
  };
}
