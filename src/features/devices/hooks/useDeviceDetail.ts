import { useEffect, useState } from "react";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";

export function useDeviceDetail(deviceId: number) {
  const [device, setDevice] = useState<Device | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    void devicesApi
      .get(deviceId)
      .then((next) => {
        if (!cancelled) {
          setDevice(next);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setDevice(null);
          setError(err instanceof Error ? err.message : "Failed to load device");
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
  }, [deviceId]);

  return { device, loading, error, setDevice };
}
