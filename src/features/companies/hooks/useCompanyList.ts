import { useCallback, useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type {
  Company,
  CompanyListResult,
} from "@/features/companies/types/company";

export function useCompanyList() {
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<CompanyListResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await companiesApi.list({
        query: query.trim() || undefined,
        includeArchived,
        page,
        pageSize: 25,
      });
      setResult(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load companies");
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, [query, includeArchived, page]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const archive = useCallback(
    async (company: Company) => {
      await companiesApi.archive(company.id);
      await reload();
    },
    [reload],
  );

  const unarchive = useCallback(
    async (company: Company) => {
      await companiesApi.unarchive(company.id);
      await reload();
    },
    [reload],
  );

  const setDefault = useCallback(
    async (company: Company) => {
      await companiesApi.setDefault(company.id);
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
    includeArchived,
    setIncludeArchived: (value: boolean) => {
      setPage(1);
      setIncludeArchived(value);
    },
    page,
    setPage,
    result,
    loading,
    error,
    reload,
    archive,
    unarchive,
    setDefault,
  };
}
