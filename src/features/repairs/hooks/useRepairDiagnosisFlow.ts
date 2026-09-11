import { useForm } from "@tanstack/react-form";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import {
  templateItemsToResult,
  type DiagnosisResultItem,
  type DiagnosisTemplate,
} from "@/features/diagnosis/types/diagnosis";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type {
  CompleteDiagnosisMode,
  Repair,
} from "@/features/repairs/types/repair";
import { checklistHasApplicableValues } from "@/features/repairs/utils/diagnosisNotesMerge";
import {
  diagnosisFormValues,
  isDiagnosisFlowDirty,
  notesWithChecklist,
  repairCanPrintDiagnosis,
  resolveDiagnosisEstimateFields,
  retainedTemplateId,
  updateChecklistItem,
  type DiagnosisFormValues,
} from "@/features/repairs/utils/repairDiagnosisFlow";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type { ShopSettings } from "@/features/settings/types/shopSettings";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useRepairDiagnosisFlow(
  repairId: string,
  options: {
    enabled?: boolean;
    onFinalizeSuccess?: (repair: Repair) => void;
    onDraftSuccess?: (repair: Repair) => void;
  } = {},
) {
  const { enabled = true, onFinalizeSuccess, onDraftSuccess } = options;
  const { t } = useI18n();
  const [repair, setRepair] = useState<Repair | null>(null);
  const [shopSettings, setShopSettings] = useState<ShopSettings | null>(null);
  const [templates, setTemplates] = useState<DiagnosisTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [checklistItems, setChecklistItems] = useState<DiagnosisResultItem[]>(
    [],
  );
  const [loading, setLoading] = useState(true);
  const [loadingTemplate, setLoadingTemplate] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [estimateLocked, setEstimateLocked] = useState(false);

  // Stable empty defaults — `useForm` calls `update(opts)` every render; if
  // `reset()` also rewrites `options.defaultValues`, the next update wipes
  // prefilled estimate fields back to "".
  const emptyDefaults = useMemo<DiagnosisFormValues>(
    () => ({
      diagnosisNotes: "",
      expectedPickupAt: "",
      estimateMajor: "",
      estimateDiscountPercent: "",
    }),
    [],
  );

  const form = useForm({
    defaultValues: emptyDefaults,
  });

  const applyRepairToForm = useCallback(
    (nextRepair: Repair) => {
      setEstimateLocked(nextRepair.estimateBaseCents != null);
      form.reset(diagnosisFormValues(nextRepair), {
        keepDefaultValues: true,
      });
    },
    [form],
  );

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [nextRepair, nextShop, templateList] = await Promise.all([
        repairsApi.get(repairId),
        settingsApi.getShopSettings(),
        diagnosisTemplatesApi.list({ page: 1, pageSize: 100 }),
      ]);
      setRepair(nextRepair);
      setShopSettings(nextShop);
      setTemplates(templateList.items);
      applyRepairToForm(nextRepair);
      setSelectedTemplateId((prev) =>
        retainedTemplateId(prev, templateList.items),
      );
      setChecklistItems([]);
    } catch (err) {
      setRepair(null);
      setShopSettings(null);
      setTemplates([]);
      setError(
        err instanceof Error
          ? err.message
          : t("repairs.diagnosisFlow.errors.loadFailed"),
      );
    } finally {
      setLoading(false);
    }
  }, [applyRepairToForm, repairId, t]);

  const repairRef = useRef(repair);
  repairRef.current = repair;
  const checklistRef = useRef(checklistItems);
  checklistRef.current = checklistItems;
  const enabledRef = useRef(enabled);
  enabledRef.current = enabled;

  const silentRefresh = useCallback(async () => {
    if (!enabledRef.current) {
      return;
    }
    if (
      isDiagnosisFlowDirty(
        form.state.values,
        repairRef.current,
        checklistRef.current,
      )
    ) {
      return;
    }
    try {
      const [nextRepair, nextShop, templateList] = await Promise.all([
        repairsApi.get(repairId),
        settingsApi.getShopSettings(),
        diagnosisTemplatesApi.list({ page: 1, pageSize: 100 }),
      ]);
      if (
        isDiagnosisFlowDirty(
          form.state.values,
          repairRef.current,
          checklistRef.current,
        )
      ) {
        return;
      }
      setRepair(nextRepair);
      setShopSettings(nextShop);
      setTemplates(templateList.items);
      applyRepairToForm(nextRepair);
      setSelectedTemplateId((prev) =>
        retainedTemplateId(prev, templateList.items),
      );
    } catch {
      // Keep the last good repair and form values.
    }
  }, [applyRepairToForm, form, repairId]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    void reload();
  }, [enabled, reload]);

  useSyncApplied(() => {
    void silentRefresh();
  });

  const canPrint = repair ? repairCanPrintDiagnosis(repair) : false;

  const loadTemplateChecklist = useCallback(async () => {
    const templateId = selectedTemplateId.trim();
    if (!templateId) {
      setError(t("repairs.diagnosisFlow.errors.selectTemplateFirst"));
      return;
    }
    setLoadingTemplate(true);
    setError(null);
    setSuccess(null);
    try {
      const template = await diagnosisTemplatesApi.get(templateId);
      setChecklistItems(templateItemsToResult(template.body.items));
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : t("repairs.diagnosisFlow.errors.loadTemplateFailed"),
      );
    } finally {
      setLoadingTemplate(false);
    }
  }, [selectedTemplateId, t]);

  const setChecklistItemValue = useCallback(
    (index: number, value: boolean | string) => {
      setSuccess(null);
      setChecklistItems((prev) => updateChecklistItem(prev, index, value));
    },
    [],
  );

  const applyChecklistToNotes = useCallback(() => {
    if (!checklistHasApplicableValues(checklistItems)) {
      setError(t("repairs.diagnosisFlow.errors.nothingToApply"));
      return;
    }
    setError(null);
    const next = notesWithChecklist(
      form.state.values.diagnosisNotes,
      checklistItems,
    ).notes;
    form.setFieldValue("diagnosisNotes", next);
    setChecklistItems([]);
    setSuccess(t("repairs.diagnosisFlow.applyToNotesSuccess"));
  }, [checklistItems, form, t]);

  const resolveNotesForSave = useCallback(() => {
    const result = notesWithChecklist(
      form.state.values.diagnosisNotes,
      checklistItems,
    );
    if (result.applied) {
      const notes = result.notes;
      form.setFieldValue("diagnosisNotes", notes);
      setChecklistItems([]);
      return notes;
    }
    return result.notes;
  }, [checklistItems, form]);

  const resolveEstimateForSave = useCallback(() => {
    const result = resolveDiagnosisEstimateFields(
      form.state.values.estimateMajor,
      form.state.values.estimateDiscountPercent,
      estimateLocked,
    );
    if (!result.ok) {
      if (result.reason === "required") {
        throw new Error(t("repairs.diagnosisFlow.errors.estimateRequired"));
      }
      if (result.reason === "discountNeedsPrice") {
        throw new Error(
          t("repairs.intake.estimate.validation.discountNeedsPrice"),
        );
      }
      if (result.reason === "invalidDiscount") {
        throw new Error(t("repairs.diagnosisFlow.estimatePreviewInvalid"));
      }
      throw new Error(t("repairs.diagnosisFlow.errors.estimateInvalid"));
    }
    return result;
  }, [estimateLocked, form, t]);

  const save = useCallback(
    async (mode: CompleteDiagnosisMode) => {
      setSaving(true);
      setError(null);
      setSuccess(null);
      try {
        const diagnosisNotes = resolveNotesForSave().trim() || null;
        if (mode === "finalize" && !diagnosisNotes) {
          throw new Error(t("repairs.diagnosisFlow.errors.notesRequired"));
        }
        const estimate = resolveEstimateForSave();
        const expectedPickupAt =
          form.state.values.expectedPickupAt.trim() || null;

        const result = await repairsApi.completeDiagnosis({
          repairId,
          mode,
          diagnosisNotes,
          expectedPickupAt,
          estimateBaseCents: estimate.estimateBaseCents,
          estimateDiscountBps: estimate.estimateDiscountBps,
        });
        setRepair(result.repair);
        applyRepairToForm(result.repair);
        setSuccess(
          mode === "finalize"
            ? t("repairs.diagnosisFlow.finalizeSuccess")
            : t("repairs.diagnosisFlow.draftSuccess"),
        );
        if (mode === "finalize") {
          onFinalizeSuccess?.(result.repair);
        } else {
          onDraftSuccess?.(result.repair);
        }
      } catch (err) {
        setError(
          err instanceof Error
            ? err.message
            : t("repairs.diagnosisFlow.errors.saveFailed"),
        );
      } finally {
        setSaving(false);
      }
    },
    [
      form,
      onDraftSuccess,
      onFinalizeSuccess,
      repairId,
      resolveEstimateForSave,
      resolveNotesForSave,
      t,
    ],
  );

  return {
    repair,
    shopSettings,
    templates,
    selectedTemplateId,
    setSelectedTemplateId,
    checklistItems,
    setChecklistItemValue,
    loading,
    loadingTemplate,
    saving,
    error,
    success,
    estimateLocked,
    canPrint,
    form,
    loadTemplateChecklist,
    applyChecklistToNotes,
    saveDraft: () => void save("draft"),
    finalize: () => void save("finalize"),
    reload,
  };
}

export type RepairDiagnosisFlowState = ReturnType<
  typeof useRepairDiagnosisFlow
>;
