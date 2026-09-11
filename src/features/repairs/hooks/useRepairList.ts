import { useCallback, useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type {
  RepairListResult,
  RepairStatus,
} from "@/features/repairs/types/repair";
import { REPAIR_STATUSES } from "@/features/repairs/types/repair";
import { DEFAULT_PAGE_SIZE } from "@/shared/constants/pagination";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

type Options = {
  customerId?: string;
  deviceId?: string;
  companyId?: string;
  initialStatus?: RepairStatus | "";
};

function parseStatus(value: string | undefined | null): RepairStatus | "" {
  if (!value) {
    return "";
  }
  return (REPAIR_STATUSES as readonly string[]).includes(value)
    ? (value as RepairStatus)
    : "";
}

export function useRepairList(options: Options = {}) {
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState<RepairStatus | "">(() =>
    parseStatus(options.initialStatus),
  );
  const [page, setPage] = useState(1);
  const [pageSize, setPageSizeState] = useState(DEFAULT_PAGE_SIZE);
  const [result, setResult] = useState<RepairListResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const next = parseStatus(options.initialStatus);
    setStatus(next);
    setPage(1);
  }, [options.initialStatus]);

  const reload = useCallback(
    async (reloadOptions?: { silent?: boolean }) => {
      const silent = reloadOptions?.silent === true;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const next = await repairsApi.list({
          query: query.trim() || undefined,
          customerId: options.customerId,
          deviceId: options.deviceId,
          companyId: options.companyId,
          status: status || undefined,
          page,
          pageSize,
        });
        setResult(next);
        setError(null);
      } catch (err) {
        if (!silent) {
          setError(
            err instanceof Error ? err.message : "Failed to load repairs",
          );
          setResult(null);
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [
      query,
      status,
      page,
      pageSize,
      options.customerId,
      options.deviceId,
      options.companyId,
    ],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

  return {
    query,
    setQuery: (value: string) => {
      setPage(1);
      setQuery(value);
    },
    status,
    setStatus: (value: RepairStatus | "") => {
      setPage(1);
      setStatus(value);
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
  };
}
