import { useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";

export function useCustomerDetail(id: number) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    void customersApi
      .get(id)
      .then((data) => {
        if (!cancelled) {
          setCustomer(data);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load customer");
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
  }, [id]);

  return { customer, loading, error, setCustomer };
}
