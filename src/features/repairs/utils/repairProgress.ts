import type { RepairStatus } from "@/features/repairs/types/repair";

/** Linear lifecycle steps shown on the repair detail progress bar (excludes cancelled). */
export const REPAIR_PROGRESS_STEPS = [
  "received",
  "signedIntake",
  "diagnosis",
  "waiting_customer",
  "waiting_part",
  "in_repair",
  "signedSummary",
  "collected",
] as const;

export type RepairProgressStep = (typeof REPAIR_PROGRESS_STEPS)[number];

const PROGRESS_STEP_INDEX = Object.fromEntries(
  REPAIR_PROGRESS_STEPS.map((step, index) => [step, index]),
) as Record<RepairProgressStep, number>;

/** Index of a named progress step (for comparisons). */
export function repairProgressStepIndex(step: RepairProgressStep): number {
  return PROGRESS_STEP_INDEX[step];
}

/** Current progress index for a repair status. */
export function repairProgressIndex(status: RepairStatus): number {
  if (status === "cancelled") {
    return -1;
  }

  switch (status) {
    case "received":
      return PROGRESS_STEP_INDEX.signedIntake;
    case "diagnosis":
      return PROGRESS_STEP_INDEX.diagnosis;
    case "waiting_customer":
      return PROGRESS_STEP_INDEX.waiting_customer;
    case "waiting_part":
      return PROGRESS_STEP_INDEX.waiting_part;
    case "in_repair":
      return PROGRESS_STEP_INDEX.in_repair;
    case "ready":
      return PROGRESS_STEP_INDEX.signedSummary;
    case "awaiting_pickup":
      return PROGRESS_STEP_INDEX.collected;
    case "collected":
      return PROGRESS_STEP_INDEX.collected;
    default:
      return 0;
  }
}

export function repairProgressPercent(status: RepairStatus): number {
  if (status === "cancelled") {
    return 0;
  }
  const index = repairProgressIndex(status);
  return Math.round(((index + 1) / REPAIR_PROGRESS_STEPS.length) * 100);
}

/** @deprecated Use RepairProgressStep */
export type RepairProgressStatus = RepairProgressStep;
