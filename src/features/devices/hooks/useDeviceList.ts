import { useCallback, useEffect, useState } from "react";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device, DeviceListResult } from "@/features/devices/types/device";
import { DEFAULT_PAGE_SIZE } from "@/shared/constants/pagination";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

type Options = {
  customerId?: string;
};

export function useDeviceList(options: Options = {}) {
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSizeState] = useState(DEFAULT_PAGE_SIZE);
  const [result, setResult] = useState<DeviceListResult | null>(null);
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
        const next = await devicesApi.list({
          query: query.trim() || undefined,
          customerId: options.customerId,
          includeArchived,
          page,
          pageSize,
        });
        setResult(next);
        setError(null);
      } catch (err) {
        if (!silent) {
          setError(
            err instanceof Error ? err.message : "Failed to load devices",
          );
          setResult(null);
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [query, includeArchived, page, pageSize, options.customerId],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

  const archive = useCallback(
    async (device: Device) => {
      await devicesApi.archive(device.id);
      await reload();
    },
    [reload],
  );

  const unarchive = useCallback(
    async (device: Device) => {
      await devicesApi.unarchive(device.id);
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
    pageSize,
    setPageSize: (size: number) => {
      setPage(1);
      setPageSizeState(size);
    },
    result,
    loading,
    error,
    reload,
    archive,
    unarchive,
  };
}
