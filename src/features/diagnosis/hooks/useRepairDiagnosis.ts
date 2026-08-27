import { useCallback, useEffect, useState } from "react";
import {
  diagnosisTemplatesApi,
  repairDiagnosisApi,
} from "@/features/diagnosis/api/diagnosisApi";
import {
  templateItemsToResult,
  type DiagnosisResultItem,
  type DiagnosisTemplate,
  type RepairDiagnosis,
} from "@/features/diagnosis/types/diagnosis";

export function useRepairDiagnosis(repairId: number) {
  const [diagnosis, setDiagnosis] = useState<RepairDiagnosis | null>(null);
  const [templates, setTemplates] = useState<DiagnosisTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [draftItems, setDraftItems] = useState<DiagnosisResultItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [nextDiagnosis, templateList] = await Promise.all([
        repairDiagnosisApi.get(repairId),
        diagnosisTemplatesApi.list({ page: 1, pageSize: 100 }),
      ]);
      setDiagnosis(nextDiagnosis);
      setDraftItems(nextDiagnosis?.result.items ?? []);
      setTemplates(templateList.items);
      setSelectedTemplateId((prev) => {
        if (
          prev &&
          templateList.items.some((template) => String(template.id) === prev)
        ) {
          return prev;
        }
        return templateList.items[0] ? String(templateList.items[0].id) : "";
      });
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to load diagnosis",
      );
      setDiagnosis(null);
      setDraftItems([]);
      setTemplates([]);
    } finally {
      setLoading(false);
    }
  }, [repairId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const applyTemplate = useCallback(async () => {
    const templateId = Number(selectedTemplateId);
    if (!Number.isFinite(templateId) || templateId <= 0) {
      setError("Select a template first.");
      return;
    }

    setApplying(true);
    setError(null);
    setSuccess(null);
    try {
      const template = await diagnosisTemplatesApi.get(templateId);
      const result = { items: templateItemsToResult(template.body.items) };
      const saved = await repairDiagnosisApi.upsert({
        repairId,
        templateId,
        result,
      });
      setDiagnosis(saved);
      setDraftItems(saved.result.items);
      setSuccess("Template applied. Fill the checklist and save.");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to apply template",
      );
    } finally {
      setApplying(false);
    }
  }, [repairId, selectedTemplateId]);

  const setItemValue = useCallback(
    (index: number, value: boolean | string) => {
      setSuccess(null);
      setDraftItems((prev) =>
        prev.map((item, i) => (i === index ? { ...item, value } : item)),
      );
    },
    [],
  );

  const save = useCallback(async () => {
    if (draftItems.length === 0) {
      setError("Nothing to save. Apply a template first.");
      return;
    }

    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const saved = await repairDiagnosisApi.upsert({
        repairId,
        templateId: diagnosis?.templateId ?? null,
        result: { items: draftItems },
      });
      setDiagnosis(saved);
      setDraftItems(saved.result.items);
      setSuccess("Diagnosis saved.");

      // Confirm persistence (surfaces get/upsert IPC mismatches immediately).
      const verified = await repairDiagnosisApi.get(repairId);
      if (!verified) {
        setError(
          "Saved, but could not reload diagnosis. Try refreshing the page.",
        );
        setSuccess(null);
      } else {
        setDiagnosis(verified);
        setDraftItems(verified.result.items);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save diagnosis");
    } finally {
      setSaving(false);
    }
  }, [repairId, diagnosis?.templateId, draftItems]);

  return {
    diagnosis,
    templates,
    selectedTemplateId,
    setSelectedTemplateId,
    draftItems,
    setItemValue,
    loading,
    saving,
    applying,
    error,
    success,
    applyTemplate,
    save,
  };
}
