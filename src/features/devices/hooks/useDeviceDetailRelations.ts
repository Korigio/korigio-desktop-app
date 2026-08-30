import { useCallback, useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useDeviceDetailRelations(device: Device | null) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(false);

  const customerId = device?.customerId;

  const load = useCallback(
    async (silent: boolean) => {
      if (!customerId) {
        if (!silent) {
          setCustomer(null);
          setLoading(false);
        }
        return;
      }

      if (!silent) {
        setLoading(true);
      }

      try {
        const nextCustomer = await customersApi.get(customerId);
        setCustomer(nextCustomer);
      } catch {
        if (!silent) {
          setCustomer(null);
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [customerId],
  );

  useEffect(() => {
    if (!customerId) {
      setCustomer(null);
      setLoading(false);
      return;
    }
    void load(false);
  }, [customerId, load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { customer, loading };
}
