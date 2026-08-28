import { useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";

export function useDeviceDetailRelations(device: Device | null) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!device) {
      setCustomer(null);
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);

    void customersApi
      .get(device.customerId)
      .then((nextCustomer) => {
        if (!cancelled) {
          setCustomer(nextCustomer);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setCustomer(null);
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
  }, [device]);

  return { customer, loading };
}
