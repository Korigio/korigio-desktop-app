import { useCallback, useEffect, useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import type {
  DiagnosisTemplate,
  DiagnosisTemplateListResult,
} from "@/features/diagnosis/types/diagnosis";

export function useDiagnosisTemplateList() {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<DiagnosisTemplateListResult | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await diagnosisTemplatesApi.list({
        query: query.trim() || undefined,
        page,
        pageSize: 25,
      });
      setResult(next);
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to load diagnosis templates",
      );
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, [query, page]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const remove = useCallback(
    async (template: DiagnosisTemplate) => {
      await diagnosisTemplatesApi.delete(template.id);
      await reload();
    },
    [reload],
  );

  return {
    query,
    setQuery: (value: string) => {
      setPage(1);
      setQuery(value);
    },
    page,
    setPage,
    result,
    loading,
    error,
    reload,
    remove,
  };
}
