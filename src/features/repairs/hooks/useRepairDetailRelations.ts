import { useEffect, useState } from "react";
import { companiesApi } from "@/features/companies/api/companiesApi";
import type { Company } from "@/features/companies/types/company";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";
import type { Repair } from "@/features/repairs/types/repair";

export function useRepairDetailRelations(repair: Repair | null) {
  const [customer, setCustomer] = useState<Customer | null>(null);
  const [device, setDevice] = useState<Device | null>(null);
  const [company, setCompany] = useState<Company | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!repair) {
      setCustomer(null);
      setDevice(null);
      setCompany(null);
      setLoading(false);
      return;
    }

    const { customerId, deviceId, companyId } = repair;
    let cancelled = false;
    setLoading(true);

    void Promise.all([
      customersApi.get(customerId),
      devicesApi.get(deviceId),
      companyId ? companiesApi.get(companyId) : Promise.resolve(null),
    ])
      .then(([nextCustomer, nextDevice, nextCompany]) => {
        if (cancelled) {
          return;
        }
        setCustomer(nextCustomer);
        setDevice(nextDevice);
        setCompany(nextCompany);
      })
      .catch(() => {
        if (!cancelled) {
          setCustomer(null);
          setDevice(null);
          setCompany(null);
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
  }, [repair]);

  return { customer, device, company, loading };
}
