import { useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";

export function useCompanyDetail(id: number) {
  const [company, setCompany] = useState<Company | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    void companiesApi
      .get(id)
      .then((data) => {
        if (!cancelled) {
          setCompany(data);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load company");
          setCompany(null);
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

  return { company, loading, error, setCompany };
}
