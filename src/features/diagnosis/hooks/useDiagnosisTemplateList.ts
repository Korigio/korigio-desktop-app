import { useCallback, useEffect, useState } from "react";
import { diagnosisTemplatesApi } from "@/features/diagnosis/api/diagnosisApi";
import type {
  DiagnosisTemplate,
  DiagnosisTemplateListResult,
} from "@/features/diagnosis/types/diagnosis";
import { DEFAULT_PAGE_SIZE } from "@/shared/constants/pagination";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useDiagnosisTemplateList() {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSizeState] = useState(DEFAULT_PAGE_SIZE);
  const [result, setResult] = useState<DiagnosisTemplateListResult | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(
    async (reloadOptions?: { silent?: boolean }) => {
      const silent = reloadOptions?.silent === true;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const next = await diagnosisTemplatesApi.list({
          query: query.trim() || undefined,
          page,
          pageSize,
        });
        setResult(next);
        setError(null);
      } catch (err) {
        if (!silent) {
          setError(
            err instanceof Error
              ? err.message
              : "Failed to load diagnosis templates",
          );
          setResult(null);
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [query, page, pageSize],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

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
    pageSize,
    setPageSize: (size: number) => {
      setPage(1);
      setPageSizeState(size);
    },
    result,
    loading,
    error,
    reload,
    remove,
  };
}
