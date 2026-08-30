import { useCallback, useEffect, useRef, useState } from "react";
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
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

function diagnosisDraftDirty(
  draftItems: DiagnosisResultItem[],
  diagnosis: RepairDiagnosis | null,
): boolean {
  return (
    JSON.stringify(draftItems) !==
    JSON.stringify(diagnosis?.result.items ?? [])
  );
}

export function useRepairDiagnosis(repairId: string) {
  const { t } = useI18n();
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

  const draftItemsRef = useRef(draftItems);
  draftItemsRef.current = draftItems;
  const diagnosisRef = useRef(diagnosis);
  diagnosisRef.current = diagnosis;

  const silentRefresh = useCallback(async () => {
    if (diagnosisDraftDirty(draftItemsRef.current, diagnosisRef.current)) {
      return;
    }
    try {
      const [nextDiagnosis, templateList] = await Promise.all([
        repairDiagnosisApi.get(repairId),
        diagnosisTemplatesApi.list({ page: 1, pageSize: 100 }),
      ]);
      if (diagnosisDraftDirty(draftItemsRef.current, diagnosisRef.current)) {
        return;
      }
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
    } catch {
      // Keep the last good diagnosis and drafts.
    }
  }, [repairId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void silentRefresh();
  });

  const applyTemplate = useCallback(async () => {
    const templateId = selectedTemplateId.trim();
    if (!templateId) {
      setError(t("diagnosis.selectTemplateFirst"));
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
      setSuccess(t("diagnosis.applySuccess"));
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to apply template",
      );
    } finally {
      setApplying(false);
    }
  }, [repairId, selectedTemplateId, t]);

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
      setError(t("diagnosis.nothingToSave"));
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

      const verified = await repairDiagnosisApi.get(repairId);
      if (!verified) {
        setError(t("diagnosis.saveReloadFailed"));
      } else {
        setDiagnosis(verified);
        setDraftItems(verified.result.items);
        setSuccess(t("diagnosis.saveSuccess"));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save diagnosis");
    } finally {
      setSaving(false);
    }
  }, [repairId, diagnosis?.templateId, draftItems, t]);

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
