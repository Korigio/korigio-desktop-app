import { useCallback, useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type {
  Customer,
  CustomerListResult,
} from "@/features/customers/types/customer";

export function useCustomerList() {
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<CustomerListResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await customersApi.list({
        query: query.trim() || undefined,
        includeArchived,
        page,
        pageSize: 25,
      });
      setResult(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load customers");
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, [query, includeArchived, page]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const archive = useCallback(
    async (customer: Customer) => {
      await customersApi.archive(customer.id);
      await reload();
    },
    [reload],
  );

  const unarchive = useCallback(
    async (customer: Customer) => {
      await customersApi.unarchive(customer.id);
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
  };
}
