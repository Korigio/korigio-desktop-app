import { useCallback, useEffect, useState } from "react";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device, DeviceListResult } from "@/features/devices/types/device";

type Options = {
  customerId?: number;
};

export function useDeviceList(options: Options = {}) {
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<DeviceListResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await devicesApi.list({
        query: query.trim() || undefined,
        customerId: options.customerId,
        includeArchived,
        page,
        pageSize: 25,
      });
      setResult(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load devices");
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, [query, includeArchived, page, options.customerId]);

  useEffect(() => {
    void reload();
  }, [reload]);

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
    result,
    loading,
    error,
    reload,
    archive,
    unarchive,
  };
}
