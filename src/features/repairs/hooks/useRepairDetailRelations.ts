import { useCallback, useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";
import type { Repair } from "@/features/repairs/types/repair";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useRepairDetailRelations(repair: Repair | null) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [device, setDevice] = useState<Device | null>(null);
  const [company, setCompany] = useState<Company | null>(null);
  const [loading, setLoading] = useState(false);

  const customerId = repair?.customerId;
  const deviceId = repair?.deviceId;
  const companyId = repair?.companyId ?? null;

  const load = useCallback(
    async (silent: boolean) => {
      if (!customerId || !deviceId) {
        if (!silent) {
          setCustomer(null);
          setDevice(null);
          setCompany(null);
          setLoading(false);
        }
        return;
      }

      if (!silent) {
        setLoading(true);
      }

      try {
        const [nextCustomer, nextDevice, nextCompany] = await Promise.all([
          customersApi.get(customerId),
          devicesApi.get(deviceId),
          companyId ? companiesApi.get(companyId) : Promise.resolve(null),
        ]);
        setCustomer(nextCustomer);
        setDevice(nextDevice);
        setCompany(nextCompany);
      } catch {
        if (!silent) {
          setCustomer(null);
          setDevice(null);
          setCompany(null);
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [customerId, deviceId, companyId],
  );

  useEffect(() => {
    if (!customerId || !deviceId) {
      setCustomer(null);
      setDevice(null);
      setCompany(null);
      setLoading(false);
      return;
    }
    void load(false);
  }, [customerId, deviceId, companyId, load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { customer, device, company, loading };
}
