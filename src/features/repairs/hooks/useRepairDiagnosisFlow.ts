import { useForm } from "@tanstack/react-form";
import { useCallback, useEffect, useState } from "react";
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
import {
  appendChecklistToNotes,
  checklistHasApplicableValues,
} from "@/features/repairs/utils/diagnosisNotesMerge";
import {
  centsToMajorInput,
  parseMajorToCents,
} from "@/features/repairs/utils/money";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type { ShopSettings } from "@/features/settings/types/shopSettings";
import { useI18n } from "@/shared/hooks/useI18n";

type FormValues = {
  diagnosisNotes: string;
  expectedPickupAt: string;
  estimateMajor: string;
};

function toFormValues(repair: Repair): FormValues {
  return {
    diagnosisNotes: repair.diagnosisNotes ?? "",
    expectedPickupAt: repair.expectedPickupAt ?? "",
    estimateMajor:
      repair.estimateBaseCents != null
        ? centsToMajorInput(repair.estimateBaseCents)
        : "",
  };
}

function repairCanPrint(repair: Repair): boolean {
  return Boolean(
    repair.diagnosisNotes?.trim() && repair.estimateBaseCents != null,
  );
}

export function useRepairDiagnosisFlow(
  repairId: number,
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

  const form = useForm({
    defaultValues: {
      diagnosisNotes: "",
      expectedPickupAt: "",
      estimateMajor: "",
    } satisfies FormValues,
  });

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
      setEstimateLocked(nextRepair.estimateBaseCents != null);
      form.reset(toFormValues(nextRepair));
      setSelectedTemplateId((prev) => {
        if (
          prev &&
          templateList.items.some((template) => String(template.id) === prev)
        ) {
          return prev;
        }
        return templateList.items[0] ? String(templateList.items[0].id) : "";
      });
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
  }, [form, repairId, t]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    void reload();
  }, [enabled, reload]);

  const canPrint = repair ? repairCanPrint(repair) : false;

  const loadTemplateChecklist = useCallback(async () => {
    const templateId = Number(selectedTemplateId);
    if (!Number.isFinite(templateId) || templateId <= 0) {
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
      setChecklistItems((prev) =>
        prev.map((item, i) => (i === index ? { ...item, value } : item)),
      );
    },
    [],
  );

  const applyChecklistToNotes = useCallback(() => {
    if (!checklistHasApplicableValues(checklistItems)) {
      setError(t("repairs.diagnosisFlow.errors.nothingToApply"));
      return;
    }
    setError(null);
    const next = appendChecklistToNotes(
      form.state.values.diagnosisNotes,
      checklistItems,
    );
    form.setFieldValue("diagnosisNotes", next);
    setChecklistItems([]);
    setSuccess(t("repairs.diagnosisFlow.applyToNotesSuccess"));
  }, [checklistItems, form, t]);

  const resolveNotesForSave = useCallback(() => {
    let notes = form.state.values.diagnosisNotes;
    if (checklistHasApplicableValues(checklistItems)) {
      notes = appendChecklistToNotes(notes, checklistItems);
      form.setFieldValue("diagnosisNotes", notes);
      setChecklistItems([]);
    }
    return notes;
  }, [checklistItems, form]);

  const resolveEstimateCents = useCallback((): number | null => {
    const raw = form.state.values.estimateMajor.trim();
    if (!raw) {
      if (estimateLocked) {
        throw new Error(t("repairs.diagnosisFlow.errors.estimateRequired"));
      }
      return null;
    }
    const cents = parseMajorToCents(raw);
    if (cents === null) {
      throw new Error(t("repairs.diagnosisFlow.errors.estimateInvalid"));
    }
    return cents;
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
        const estimateBaseCents = resolveEstimateCents();
        const expectedPickupAt =
          form.state.values.expectedPickupAt.trim() || null;

        const result = await repairsApi.completeDiagnosis({
          repairId,
          mode,
          diagnosisNotes,
          expectedPickupAt,
          estimateBaseCents,
        });
        setRepair(result.repair);
        setEstimateLocked(result.repair.estimateBaseCents != null);
        form.reset(toFormValues(result.repair));
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
    [form, onDraftSuccess, onFinalizeSuccess, repairId, resolveEstimateCents, resolveNotesForSave, t],
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

export type RepairDiagnosisFlowState = ReturnType<typeof useRepairDiagnosisFlow>;
