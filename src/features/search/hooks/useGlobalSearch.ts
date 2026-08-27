import { useEffect, useState } from "react";
import { searchApi } from "@/features/search/api/searchApi";
import {
  EMPTY_SEARCH_RESULT,
  type GlobalSearchResult,
} from "@/features/search/types/search";
import { useDebouncedValue } from "@/shared/hooks/useDebouncedValue";

const DEBOUNCE_MS = 250;

export function useGlobalSearch() {
  const [query, setQuery] = useState("");
  const debouncedQuery = useDebouncedValue(query, DEBOUNCE_MS);
  const [result, setResult] = useState<GlobalSearchResult>(EMPTY_SEARCH_RESULT);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    void searchApi
      .globalSearch({ query: debouncedQuery })
      .then((next) => {
        if (!cancelled) {
          setResult(next);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(
            err instanceof Error ? err.message : "Failed to search",
          );
          setResult(EMPTY_SEARCH_RESULT);
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
  }, [debouncedQuery]);

  return {
    query,
    setQuery,
    debouncedQuery,
    result,
    loading,
    error,
  };
}
