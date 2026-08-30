import { useCallback, useEffect, useRef, useState } from "react";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useDeviceDetail(deviceId: string) {
  const [device, setDevice] = useState<Device | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const deviceIdRef = useRef(deviceId);
  deviceIdRef.current = deviceId;

  const load = useCallback(async (silent: boolean) => {
    const id = deviceId;
    if (!silent) {
      setLoading(true);
      setError(null);
    }
    try {
      const next = await devicesApi.get(id);
      if (deviceIdRef.current !== id) {
        return;
      }
      setDevice(next);
      setError(null);
    } catch (err) {
      if (deviceIdRef.current !== id) {
        return;
      }
      if (!silent) {
        setDevice(null);
        setError(err instanceof Error ? err.message : "Failed to load device");
      }
    } finally {
      if (deviceIdRef.current === id && !silent) {
        setLoading(false);
      }
    }
  }, [deviceId]);

  useEffect(() => {
    void load(false);
  }, [load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { device, loading, error, setDevice };
}
