import { useCallback, useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type {
  RepairListResult,
  RepairStatus,
} from "@/features/repairs/types/repair";
import { REPAIR_STATUSES } from "@/features/repairs/types/repair";

type Options = {
  customerId?: number;
  deviceId?: number;
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
  const [status, setStatus] = useState<RepairStatus | "">(
    () => parseStatus(options.initialStatus),
  );
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<RepairListResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const next = parseStatus(options.initialStatus);
    setStatus(next);
    setPage(1);
  }, [options.initialStatus]);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await repairsApi.list({
        query: query.trim() || undefined,
        customerId: options.customerId,
        deviceId: options.deviceId,
        status: status || undefined,
        page,
        pageSize: 25,
      });
      setResult(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load repairs");
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, [query, status, page, options.customerId, options.deviceId]);

  useEffect(() => {
    void reload();
  }, [reload]);

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
    result,
    loading,
    error,
    reload,
  };
}
