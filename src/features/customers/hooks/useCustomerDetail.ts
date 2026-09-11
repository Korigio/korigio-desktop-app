import { useCallback, useEffect, useRef, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useCustomerDetail(id: string) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const idRef = useRef(id);
  idRef.current = id;

  const load = useCallback(
    async (silent: boolean) => {
      const requestedId = id;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const data = await customersApi.get(requestedId);
        if (idRef.current !== requestedId) {
          return;
        }
        setCustomer(data);
        setError(null);
      } catch (err) {
        if (idRef.current !== requestedId) {
          return;
        }
        if (!silent) {
          setError(
            err instanceof Error ? err.message : "Failed to load customer",
          );
          setCustomer(null);
        }
      } finally {
        if (idRef.current === requestedId && !silent) {
          setLoading(false);
        }
      }
    },
    [id],
  );

  useEffect(() => {
    void load(false);
  }, [load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { customer, loading, error, setCustomer };
}
