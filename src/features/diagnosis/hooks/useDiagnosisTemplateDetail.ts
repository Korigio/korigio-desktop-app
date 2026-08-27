import { useEffect, useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import type { DiagnosisTemplate } from "@/features/diagnosis/types/diagnosis";

export function useDiagnosisTemplateDetail(id: number) {
  const [template, setTemplate] = useState<DiagnosisTemplate | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    void diagnosisTemplatesApi
      .get(id)
      .then((data) => {
        if (!cancelled) {
          setTemplate(data);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(
            err instanceof Error
              ? err.message
              : "Failed to load diagnosis template",
          );
          setTemplate(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [id]);

  return { template, loading, error };
}
