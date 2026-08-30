import { useCallback, useEffect, useRef, useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import type { DiagnosisTemplate } from "@/features/diagnosis/types/diagnosis";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useDiagnosisTemplateDetail(id: string) {
  const [template, setTemplate] = useState<DiagnosisTemplate | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const idRef = useRef(id);
  idRef.current = id;

  const load = useCallback(async (silent: boolean) => {
    const requestedId = id;
    if (!silent) {
      setLoading(true);
      setError(null);
    }
    try {
      const data = await diagnosisTemplatesApi.get(requestedId);
      if (idRef.current !== requestedId) {
        return;
      }
      setTemplate(data);
      setError(null);
    } catch (err) {
      if (idRef.current !== requestedId) {
        return;
      }
      if (!silent) {
        setError(
          err instanceof Error
            ? err.message
            : "Failed to load diagnosis template",
        );
        setTemplate(null);
      }
    } finally {
      if (idRef.current === requestedId && !silent) {
        setLoading(false);
      }
    }
  }, [id]);

  useEffect(() => {
    void load(false);
  }, [load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { template, loading, error };
}
