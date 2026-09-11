import type {
  DiagnosisResultItem,
  DiagnosisTemplate,
} from "@/features/diagnosis/types/diagnosis";
import type { Repair } from "@/features/repairs/types/repair";
import {
  appendChecklistToNotes,
  checklistHasApplicableValues,
} from "@/features/repairs/utils/diagnosisNotesMerge";
import {
  centsToMajorInput,
  formatTaxRateBps,
} from "@/features/repairs/utils/money";
import {
  resolveIntakeEstimateFields,
  type ResolveIntakeEstimateResult,
} from "@/features/repairs/utils/repairIntake";

export type DiagnosisFormValues = {
  diagnosisNotes: string;
  expectedPickupAt: string;
  /** Major-unit list price (or legacy net when list is absent). */
  estimateMajor: string;
  /** Discount percent input (e.g. "10"); empty when unset / legacy. */
  estimateDiscountPercent: string;
};

/**
 * Prefill diagnosis estimate inputs from the repair:
 * - list present → list + discount (0 → "0")
 * - else legacy net only → list from net, discount ""
 * - else both empty
 */
export function diagnosisFormValues(repair: Repair): DiagnosisFormValues {
  const listCents = repair.estimateListCents ?? null;
  const discountBps = repair.estimateDiscountBps ?? null;
  const netCents = repair.estimateBaseCents;

  if (listCents != null) {
    return {
      diagnosisNotes: repair.diagnosisNotes ?? "",
      expectedPickupAt: repair.expectedPickupAt ?? "",
      estimateMajor: centsToMajorInput(listCents),
      estimateDiscountPercent:
        discountBps != null ? formatTaxRateBps(discountBps) : "",
    };
  }

  return {
    diagnosisNotes: repair.diagnosisNotes ?? "",
    expectedPickupAt: repair.expectedPickupAt ?? "",
    estimateMajor: netCents != null ? centsToMajorInput(netCents) : "",
    estimateDiscountPercent: "",
  };
}

export function repairCanPrintDiagnosis(repair: Repair): boolean {
  return Boolean(
    repair.diagnosisNotes?.trim() && repair.estimateBaseCents != null,
  );
}

export function isDiagnosisFlowDirty(
  formValues: DiagnosisFormValues,
  repair: Repair | null,
  checklistItems: DiagnosisResultItem[],
): boolean {
  if (checklistItems.length > 0) return true;
  if (!repair) return false;
  const saved = diagnosisFormValues(repair);
  return (
    formValues.diagnosisNotes !== saved.diagnosisNotes ||
    formValues.expectedPickupAt !== saved.expectedPickupAt ||
    formValues.estimateMajor !== saved.estimateMajor ||
    formValues.estimateDiscountPercent !== saved.estimateDiscountPercent
  );
}

export function retainedTemplateId(
  currentId: string,
  templates: DiagnosisTemplate[],
): string {
  if (
    currentId &&
    templates.some((template) => String(template.id) === currentId)
  )
    return currentId;
  return templates[0] ? String(templates[0].id) : "";
}

export function updateChecklistItem(
  items: DiagnosisResultItem[],
  index: number,
  value: boolean | string,
): DiagnosisResultItem[] {
  return items.map((item, itemIndex) =>
    itemIndex === index ? { ...item, value } : item,
  );
}

export function notesWithChecklist(
  notes: string,
  items: DiagnosisResultItem[],
): { notes: string; applied: boolean } {
  if (!checklistHasApplicableValues(items)) return { notes, applied: false };
  return { notes: appendChecklistToNotes(notes, items), applied: true };
}

export type ResolveDiagnosisEstimateResult =
  ResolveIntakeEstimateResult | { ok: false; reason: "required" };

/**
 * Resolve diagnosis estimate for `complete_repair_diagnosis`
 * (`estimateBaseCents` = list, plus optional discount).
 * When locked, clearing the estimate is rejected.
 */
export function resolveDiagnosisEstimateFields(
  estimateMajor: string,
  estimateDiscountPercent: string,
  estimateLocked: boolean,
): ResolveDiagnosisEstimateResult {
  const resolved = resolveIntakeEstimateFields(
    estimateMajor,
    estimateDiscountPercent,
  );
  if (!resolved.ok) return resolved;
  if (estimateLocked && resolved.estimateBaseCents === null) {
    return { ok: false, reason: "required" };
  }
  return resolved;
}
