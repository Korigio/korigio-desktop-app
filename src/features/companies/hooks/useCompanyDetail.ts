import { useCallback, useEffect, useRef, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useCompanyDetail(id: string) {
  const [company, setCompany] = useState<Company | null>(null);
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
      const data = await companiesApi.get(requestedId);
      if (idRef.current !== requestedId) {
        return;
      }
      setCompany(data);
      setError(null);
    } catch (err) {
      if (idRef.current !== requestedId) {
        return;
      }
      if (!silent) {
        setError(err instanceof Error ? err.message : "Failed to load company");
        setCompany(null);
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

  return { company, loading, error, setCompany };
}
