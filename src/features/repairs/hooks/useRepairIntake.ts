import { useCallback, useEffect, useRef, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import { deviceLabel, type Device } from "@/features/devices/types/device";
import { repairsApi } from "@/features/repairs/api/repairsApi";

function useDebouncedValue(value: string, delayMs: number): string {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(value), delayMs);
    return () => window.clearTimeout(timer);
  }, [value, delayMs]);

  return debounced;
}

function customerLabel(customer: Customer): string {
  return customer.phone
    ? `${customer.name} · ${customer.phone}`
    : customer.name;
}

export function useRepairIntake() {
  const { t } = useI18n();
  const customerInputRef = useRef<HTMLInputElement>(null);
  const problemInputRef = useRef<HTMLTextAreaElement>(null);

  const [customerQuery, setCustomerQuery] = useState("");
  const [deviceQuery, setDeviceQuery] = useState("");
  const debouncedCustomerQuery = useDebouncedValue(customerQuery, 250);
  const debouncedDeviceQuery = useDebouncedValue(deviceQuery, 250);

  const [customer, setCustomer] = useState<Customer | null>(null);
  const [device, setDevice] = useState<Device | null>(null);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [customersLoading, setCustomersLoading] = useState(false);
  const [devicesLoading, setDevicesLoading] = useState(false);

  const [reportedProblem, setReportedProblem] = useState("");
  const [accessoriesReceived, setAccessoriesReceived] = useState("");
  const [deviceCondition, setDeviceCondition] = useState("");

  const [showQuickCreateCustomer, setShowQuickCreateCustomer] = useState(false);
  const [showQuickCreateDevice, setShowQuickCreateDevice] = useState(false);
  const [quickCustomerName, setQuickCustomerName] = useState("");
  const [quickCustomerPhone, setQuickCustomerPhone] = useState("");
  const [quickDeviceType, setQuickDeviceType] = useState("");
  const [quickDeviceSerial, setQuickDeviceSerial] = useState("");

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setCustomersLoading(true);
    void customersApi
      .list({
        query: debouncedCustomerQuery.trim() || undefined,
        includeArchived: false,
        page: 1,
        pageSize: 20,
      })
      .then((result) => {
        if (!cancelled) {
          setCustomers(result.items);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setCustomers([]);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setCustomersLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [debouncedCustomerQuery]);

  useEffect(() => {
    if (!customer) {
      setDevices([]);
      return;
    }

    let cancelled = false;
    setDevicesLoading(true);
    void devicesApi
      .list({
        query: debouncedDeviceQuery.trim() || undefined,
        customerId: customer.id,
        includeArchived: false,
        page: 1,
        pageSize: 20,
      })
      .then((result) => {
        if (!cancelled) {
          setDevices(result.items);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDevices([]);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setDevicesLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [customer, debouncedDeviceQuery]);

  const focusCustomerSearch = useCallback(() => {
    customerInputRef.current?.focus();
  }, []);

  const resetForm = useCallback(() => {
    setCustomerQuery("");
    setDeviceQuery("");
    setCustomer(null);
    setDevice(null);
    setReportedProblem("");
    setAccessoriesReceived("");
    setDeviceCondition("");
    setShowQuickCreateCustomer(false);
    setShowQuickCreateDevice(false);
    setQuickCustomerName("");
    setQuickCustomerPhone("");
    setQuickDeviceType("");
    setQuickDeviceSerial("");
    setError(null);
    focusCustomerSearch();
  }, [focusCustomerSearch]);

  const selectCustomer = useCallback((next: Customer) => {
    setCustomer(next);
    setCustomerQuery("");
    setDevice(null);
    setDeviceQuery("");
    setShowQuickCreateCustomer(false);
    setShowQuickCreateDevice(false);
  }, []);

  const clearCustomer = useCallback(() => {
    setCustomer(null);
    setDevice(null);
    setDeviceQuery("");
    setCustomerQuery("");
    focusCustomerSearch();
  }, [focusCustomerSearch]);

  const selectDevice = useCallback((next: Device) => {
    setDevice(next);
    setDeviceQuery("");
    setShowQuickCreateDevice(false);
    window.setTimeout(() => problemInputRef.current?.focus(), 0);
  }, []);

  const clearDevice = useCallback(() => {
    setDevice(null);
    setDeviceQuery("");
  }, []);

  const openQuickCreateCustomer = useCallback(() => {
    setShowQuickCreateCustomer(true);
    setQuickCustomerName(customerQuery.trim());
    setQuickCustomerPhone("");
  }, [customerQuery]);

  const openQuickCreateDevice = useCallback(() => {
    setShowQuickCreateDevice(true);
    setQuickDeviceType("");
    setQuickDeviceSerial(deviceQuery.trim());
  }, [deviceQuery]);

  const createQuickCustomer = useCallback(async () => {
    const name = quickCustomerName.trim();
    if (!name) {
      setError(t("customers.validation.nameRequired"));
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const created = await customersApi.create({
        name,
        phone: quickCustomerPhone.trim() || null,
      });
      selectCustomer(created);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create customer");
    } finally {
      setSubmitting(false);
    }
  }, [quickCustomerName, quickCustomerPhone, selectCustomer, t]);

  const createQuickDevice = useCallback(async () => {
    if (!customer) {
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const created = await devicesApi.create({
        customerId: customer.id,
        deviceType: quickDeviceType.trim() || null,
        serialNumber: quickDeviceSerial.trim() || null,
      });
      selectDevice(created);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create device");
    } finally {
      setSubmitting(false);
    }
  }, [
    customer,
    quickDeviceType,
    quickDeviceSerial,
    selectDevice,
  ]);

  const submit = useCallback(async () => {
    if (!customer) {
      setError(t("repairs.validation.customerRequired"));
      focusCustomerSearch();
      return;
    }
    if (!device) {
      setError(t("repairs.validation.deviceRequired"));
      return;
    }
    if (!reportedProblem.trim()) {
      setError(t("repairs.intake.problemRequired"));
      problemInputRef.current?.focus();
      return;
    }

    setSubmitting(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const created = await repairsApi.create({
        customerId: customer.id,
        deviceId: device.id,
        status: "received",
        reportedProblem: reportedProblem.trim(),
        accessoriesReceived: accessoriesReceived.trim() || null,
        deviceCondition: deviceCondition.trim() || null,
      });
      setSuccessMessage(created.repairNumber);
      resetForm();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create repair");
    } finally {
      setSubmitting(false);
    }
  }, [
    accessoriesReceived,
    customer,
    device,
    deviceCondition,
    focusCustomerSearch,
    reportedProblem,
    resetForm,
    t,
  ]);

  return {
    customerInputRef,
    problemInputRef,
    customerQuery,
    setCustomerQuery,
    deviceQuery,
    setDeviceQuery,
    customer,
    device,
    customers,
    devices,
    customersLoading,
    devicesLoading,
    customerLabel,
    deviceLabel,
    reportedProblem,
    setReportedProblem,
    accessoriesReceived,
    setAccessoriesReceived,
    deviceCondition,
    setDeviceCondition,
    showQuickCreateCustomer,
    setShowQuickCreateCustomer,
    showQuickCreateDevice,
    setShowQuickCreateDevice,
    quickCustomerName,
    setQuickCustomerName,
    quickCustomerPhone,
    setQuickCustomerPhone,
    quickDeviceType,
    setQuickDeviceType,
    quickDeviceSerial,
    setQuickDeviceSerial,
    submitting,
    error,
    successMessage,
    selectCustomer,
    clearCustomer,
    selectDevice,
    clearDevice,
    openQuickCreateCustomer,
    openQuickCreateDevice,
    createQuickCustomer,
    createQuickDevice,
    submit,
    resetForm,
    focusCustomerSearch,
  };
}
