import type { Repair } from "@/features/repairs/types/repair";

export function isRepairDiagnosisBlocked(repair: Repair): boolean {
  return Boolean(repair.archivedAt) || repair.status === "cancelled";
}

export function isRepairCompleted(repair: Repair): boolean {
  return repair.status === "collected";
}

export function isRepairDiagnosisLocked(repair: Repair): boolean {
  return isRepairDiagnosisBlocked(repair) || isRepairCompleted(repair);
}

/** Intake confirmed — ready to start diagnosis (status diagnosis). */
export function isDiagnosisPending(repair: Repair): boolean {
  if (isRepairDiagnosisBlocked(repair)) {
    return false;
  }
  return repair.status === "diagnosis";
}

/** New repair — optional signed intake step (status received). */
export function isIntakePending(repair: Repair): boolean {
  if (isRepairDiagnosisBlocked(repair)) {
    return false;
  }
  return repair.status === "received";
}

/** Diagnosis was finalized — same repair record, open flow to adjust findings/estimate. */
export function isDiagnosisComplete(repair: Repair): boolean {
  if (isRepairDiagnosisBlocked(repair) || isIntakePending(repair)) {
    return false;
  }
  return !isDiagnosisPending(repair);
}

export function repairDiagnosisActionKey(
  repair: Repair,
): "diagnose" | "adjustDiagnosis" | null {
  if (isRepairDiagnosisLocked(repair) || isIntakePending(repair)) {
    return null;
  }
  if (isDiagnosisPending(repair)) {
    return "diagnose";
  }
  return "adjustDiagnosis";
}
