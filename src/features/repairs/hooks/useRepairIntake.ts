import { useCallback, useEffect, useRef, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { customersApi } from "@/features/customers/api/customersApi";
import { useCustomerSearchCombobox } from "@/features/customers/hooks/useCustomerSearchCombobox";
import { devicesApi } from "@/features/devices/api/devicesApi";
import { useDeviceSearchCombobox } from "@/features/devices/hooks/useDeviceSearchCombobox";
import { repairsApi } from "@/features/repairs/api/repairsApi";

export function useRepairIntake() {
  const { t } = useI18n();
  const problemInputRef = useRef<HTMLTextAreaElement>(null);

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

  const customerSearch = useCustomerSearchCombobox();

  const deviceSearch = useDeviceSearchCombobox({
    customerId: customerSearch.selected?.id,
    onSelect: () => {
      window.setTimeout(() => problemInputRef.current?.focus(), 0);
    },
  });

  const resetDeviceSearch = deviceSearch.reset;

  useEffect(() => {
    resetDeviceSearch();
  }, [customerSearch.selected?.id, resetDeviceSearch]);

  const resetForm = useCallback(() => {
    customerSearch.reset();
    deviceSearch.reset();
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
    customerSearch.focus();
  }, [customerSearch, deviceSearch]);

  const openQuickCreateCustomer = useCallback(() => {
    setShowQuickCreateCustomer(true);
    setQuickCustomerName(customerSearch.query.trim());
    setQuickCustomerPhone("");
  }, [customerSearch.query]);

  const openQuickCreateDevice = useCallback(() => {
    setShowQuickCreateDevice(true);
    setQuickDeviceType("");
    setQuickDeviceSerial(deviceSearch.query.trim());
  }, [deviceSearch.query]);

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
      customerSearch.select(created);
      setShowQuickCreateCustomer(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create customer");
    } finally {
      setSubmitting(false);
    }
  }, [quickCustomerName, quickCustomerPhone, customerSearch, t]);

  const createQuickDevice = useCallback(async () => {
    const customer = customerSearch.selected;
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
      deviceSearch.select(created);
      setShowQuickCreateDevice(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create device");
    } finally {
      setSubmitting(false);
    }
  }, [
    customerSearch.selected,
    deviceSearch,
    quickDeviceType,
    quickDeviceSerial,
  ]);

  const submit = useCallback(async () => {
    const customer = customerSearch.selected;
    const device = deviceSearch.selected;

    if (!customer) {
      setError(t("repairs.validation.customerRequired"));
      customerSearch.focus();
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
    customerSearch,
    deviceCondition,
    deviceSearch,
    reportedProblem,
    resetForm,
    t,
  ]);

  return {
    customerSearch,
    deviceSearch,
    problemInputRef,
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
    openQuickCreateCustomer,
    openQuickCreateDevice,
    createQuickCustomer,
    createQuickDevice,
    submit,
    focusCustomerSearch: customerSearch.focus,
  };
}
