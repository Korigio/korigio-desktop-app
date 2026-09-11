import { describe, expect, it } from "vitest";
import type {
  DiagnosisResultItem,
  DiagnosisTemplate,
} from "@/features/diagnosis/types/diagnosis";
import type { Repair } from "@/features/repairs/types/repair";
import {
  diagnosisFormValues,
  isDiagnosisFlowDirty,
  notesWithChecklist,
  resolveDiagnosisEstimateFields,
  retainedTemplateId,
  updateChecklistItem,
} from "./repairDiagnosisFlow";

describe("diagnosis workflow boundaries", () => {
  it("prefills list+discount when intake list is present", () => {
    const repair = {
      diagnosisNotes: "ok",
      expectedPickupAt: null,
      estimateListCents: 10_000,
      estimateDiscountBps: 1_000,
      estimateBaseCents: 9_000,
    } as Repair;
    expect(diagnosisFormValues(repair)).toEqual({
      diagnosisNotes: "ok",
      expectedPickupAt: "",
      estimateMajor: "100",
      estimateDiscountPercent: "10",
    });
  });

  it("shows discount 0 when list is present and discount bps is 0", () => {
    const repair = {
      diagnosisNotes: null,
      expectedPickupAt: null,
      estimateListCents: 5_000,
      estimateDiscountBps: 0,
      estimateBaseCents: 5_000,
    } as Repair;
    expect(diagnosisFormValues(repair).estimateDiscountPercent).toBe("0");
  });

  it("prefills legacy net into list with empty discount", () => {
    const repair = {
      diagnosisNotes: "legacy",
      expectedPickupAt: "2026-01-01",
      estimateListCents: null,
      estimateDiscountBps: null,
      estimateBaseCents: 1_250,
    } as Repair;
    expect(diagnosisFormValues(repair)).toEqual({
      diagnosisNotes: "legacy",
      expectedPickupAt: "2026-01-01",
      estimateMajor: "12.50",
      estimateDiscountPercent: "",
    });
  });

  it("leaves estimate empty when no saved estimate", () => {
    const repair = {
      diagnosisNotes: null,
      expectedPickupAt: null,
      estimateListCents: null,
      estimateBaseCents: null,
    } as Repair;
    expect(diagnosisFormValues(repair)).toEqual({
      diagnosisNotes: "",
      expectedPickupAt: "",
      estimateMajor: "",
      estimateDiscountPercent: "",
    });
  });

  it("distinguishes untouched, changed, and checklist-dirty state", () => {
    const repair = {
      diagnosisNotes: "ok",
      expectedPickupAt: null,
      estimateListCents: 10_000,
      estimateDiscountBps: 0,
      estimateBaseCents: 10_000,
    } as Repair;
    const values = diagnosisFormValues(repair);
    expect(isDiagnosisFlowDirty(values, repair, [])).toBe(false);
    expect(
      isDiagnosisFlowDirty(
        { ...values, diagnosisNotes: "changed" },
        repair,
        [],
      ),
    ).toBe(true);
    expect(
      isDiagnosisFlowDirty(
        { ...values, estimateDiscountPercent: "5" },
        repair,
        [],
      ),
    ).toBe(true);
    expect(
      isDiagnosisFlowDirty(values, repair, [
        { id: "screen", label: "screen", kind: "checkbox", value: true },
      ]),
    ).toBe(true);
  });

  it("retains a valid template and otherwise selects the first", () => {
    const templates = [{ id: "a" }, { id: "b" }] as DiagnosisTemplate[];
    expect(retainedTemplateId("b", templates)).toBe("b");
    expect(retainedTemplateId("missing", templates)).toBe("a");
    expect(retainedTemplateId("", [])).toBe("");
  });

  it("updates checklist immutably and applies meaningful values", () => {
    const items: DiagnosisResultItem[] = [
      { id: "screen", label: "screen", kind: "checkbox", value: false },
    ];
    const updated = updateChecklistItem(items, 0, true);
    expect(updated).not.toBe(items);
    expect(updated[0]?.value).toBe(true);
    expect(notesWithChecklist("existing", updated).applied).toBe(true);
    expect(notesWithChecklist("existing", [])).toEqual({
      notes: "existing",
      applied: false,
    });
  });

  it("resolves list+discount and enforces locked estimates", () => {
    expect(resolveDiagnosisEstimateFields("", "", false)).toEqual({
      ok: true,
      estimateBaseCents: null,
      estimateDiscountBps: null,
    });
    expect(resolveDiagnosisEstimateFields("", "", true)).toEqual({
      ok: false,
      reason: "required",
    });
    expect(resolveDiagnosisEstimateFields("n/a", "", false)).toEqual({
      ok: false,
      reason: "invalidPrice",
    });
    expect(resolveDiagnosisEstimateFields("100", "10", false)).toEqual({
      ok: true,
      estimateBaseCents: 10_000,
      estimateDiscountBps: 1_000,
    });
    expect(resolveDiagnosisEstimateFields("", "10", false)).toEqual({
      ok: false,
      reason: "discountNeedsPrice",
    });
  });
});
